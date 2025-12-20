use super::package::{PackageKind, TCPPackage};
use super::tcp::{self, port_number};
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::peer::SOCKET;
use crate::operative::{Chain, OperatingKind};
use crate::transmutative::key::{KeyHolder, ToNostrKeyStr};
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// Idle client timeout.
#[allow(non_camel_case_types)]
pub const IDLE_CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// Payload read timeout.
#[allow(non_camel_case_types)]
pub const PAYLOAD_READ_TIMEOUT: Duration = Duration::from_millis(3000);

/// Payload write timeout.
#[allow(non_camel_case_types)]
pub const PAYLOAD_WRITE_TIMEOUT: Duration = Duration::from_millis(10_000);

pub async fn run(
    operating_kind: OperatingKind,
    chain: Chain,
    nns_client: &NNSClient,
    keys: Arc<KeyHolder>,
) {
    let port_number = port_number(chain);
    let addr = format!("{}:{}", "0.0.0.0", port_number);
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("{}", format!("Failed to bind {}.", addr).red());

            return;
        }
    };

    match operating_kind {
        OperatingKind::Coordinator => loop {
            let (socket_, _) = match listener.accept().await {
                Ok(conn) => (conn.0, conn.1),
                Err(_) => continue,
            };

            let socket = Arc::new(Mutex::new(socket_));

            tokio::spawn({
                let socket = Arc::clone(&socket);
                let keys = Arc::clone(&keys);

                async move {
                    handle_socket(&socket, None, operating_kind, &keys).await;
                }
            });
        },
        OperatingKind::Operator => {
            let coordinator_key = coordinator_key(chain);
            let coordinator_npub = match coordinator_key.to_npub() {
                Some(npub) => npub,
                None => return,
            };

            loop {
                match nns_client.query_address(&coordinator_npub).await {
                    Some(ip_address) => 'post_nns: loop {
                        let (socket_, socket_addr) = match listener.accept().await {
                            Ok(conn) => (conn.0, conn.1),
                            Err(_) => continue,
                        };

                        // Operator only accepts incoming connections from the coordinator.
                        if socket_addr.ip().to_string() != ip_address {
                            continue;
                        }

                        let socket = Arc::new(Mutex::new(socket_));

                        let socket_alive = Arc::new(Mutex::new(true));

                        tokio::spawn({
                            let socket = Arc::clone(&socket);
                            let socket_alive = Arc::clone(&socket_alive);
                            let keys = Arc::clone(&keys);

                            async move {
                                handle_socket(&socket, Some(&socket_alive), operating_kind, &keys)
                                    .await;
                            }
                        });

                        loop {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            let alive = {
                                let mut _alive = socket_alive.lock().await;
                                *_alive
                            };

                            if !alive {
                                break 'post_nns;
                            }
                        }
                    },
                    None => {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
        }
        OperatingKind::Node => return,
    }
}

async fn handle_socket(
    socket: &SOCKET,
    alive: Option<&Arc<Mutex<bool>>>,
    operating_kind: OperatingKind,
    keys: &KeyHolder,
) {
    loop {
        let package = {
            let mut _socket = socket.lock().await;

            // Read package kind.
            let mut package_kind_buffer = [0; 1];
            match tcp::read(
                &mut *_socket,
                &mut package_kind_buffer,
                Some(IDLE_CLIENT_TIMEOUT),
            )
            .await
            {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break, // Exit on disconnection.
                Err(tcp::TCPError::Timeout) => break, // Exit on IDLE_TIMEOUT.
                Err(_) => continue,                   // Iterate on read errors.
            }
            let package_kind = match PackageKind::from_bytecode(package_kind_buffer[0]) {
                Some(kind) => kind,
                None => continue,
            };

            // Start tracking elapsed time.
            let start = Instant::now();
            let timeout_duration = PAYLOAD_READ_TIMEOUT; // Default timeout: 3000 ms.

            // Read timestamp.
            let mut timestamp_buffer = [0; 8];
            match tcp::read(&mut *_socket, &mut timestamp_buffer, Some(timeout_duration)).await {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break, // Exit on disconnection.
                Err(tcp::TCPError::Timeout) => continue, // Iterate on PAYLOAD_READ_TIMEOUT.
                Err(_) => continue,                   // Iterate on read errors.
            }
            let timestamp = i64::from_be_bytes(timestamp_buffer);

            let remaining_time = match timeout_duration.checked_sub(start.elapsed()) {
                Some(duration) => duration,
                None => continue,
            };

            // Read payload length.
            let mut payload_len_buffer = [0; 4];
            match tcp::read(&mut *_socket, &mut payload_len_buffer, Some(remaining_time)).await {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break, // Exit on disconnection.
                Err(tcp::TCPError::Timeout) => continue, // Iterate on PAYLOAD_READ_TIMEOUT.
                Err(_) => continue,                   // Iterate on read errors.
            }
            let payload_len = u32::from_be_bytes(payload_len_buffer) as usize;

            let remaining_time = match timeout_duration.checked_sub(start.elapsed()) {
                Some(duration) => duration,
                None => continue,
            };

            // Read payload.
            let mut payload_bufer = vec![0x00u8; u32::from_be_bytes(payload_len_buffer) as usize];
            match payload_len {
                0 => continue, // Iterate on empty payload.
                _ => {
                    match tcp::read(&mut *_socket, &mut payload_bufer, Some(remaining_time)).await {
                        Ok(_) => (),
                        Err(tcp::TCPError::ConnErr) => break, // Exit on disconnection.
                        Err(tcp::TCPError::Timeout) => continue, // Iterate on PAYLOAD_READ_TIMEOUT.
                        Err(_) => continue,                   // Iterate on read errors.
                    }
                }
            }

            let package = TCPPackage::new(package_kind, timestamp, &payload_bufer);

            package
        };

        // Process the request kind.
        handle_package(package, socket, operating_kind, keys).await;
    }

    // Remove the client from the list upon disconnection.
    {
        if let Some(alive) = alive {
            let mut _alive = alive.lock().await;
            *_alive = false;
        }
    }
}

async fn handle_package(
    package: TCPPackage,
    socket: &SOCKET,
    operating_kind: OperatingKind,
    _keys: &KeyHolder,
) {
    let response_package_ = {
        match operating_kind {
            OperatingKind::Coordinator => match package.kind() {
                PackageKind::Ping => handle_ping(package.timestamp(), &package.payload()).await,
                _ => return,
            },
            OperatingKind::Operator => match package.kind() {
                PackageKind::Ping => handle_ping(package.timestamp(), &package.payload()).await,
                _ => return,
            },
            OperatingKind::Node => return,
        }
    };

    let response_package = match response_package_ {
        Some(package) => package,
        // Empty package if None.
        None => TCPPackage::new(package.kind(), package.timestamp(), &[]),
    };

    let _ = response_package
        .deliver(socket, Some(PAYLOAD_WRITE_TIMEOUT))
        .await;
}

async fn handle_ping(timestamp: i64, payload: &[u8]) -> Option<TCPPackage> {
    // Expected payload: 0x00 ping.
    if payload != &[0x00] {
        return None;
    }

    let response_package = {
        let kind = PackageKind::Ping;
        let payload = [0x01u8]; // 0x01 for pong.

        TCPPackage::new(kind, timestamp, &payload)
    };

    Some(response_package)
}
