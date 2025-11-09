use super::package::{PackageKind, TCPPackage};
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::peer::SOCKET;
use crate::transmutative::key::ToNostrKeyStr;
use crate::{inscriptive::baked, operative::Chain};
use easy_upnp::{add_ports, PortMappingProtocol, UpnpConfig};
use std::time::{Duration, Instant};
use std::{io, vec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

/// TCP response timeout.
#[allow(non_camel_case_types)]
pub const TCP_RESPONSE_TIMEOUT: u64 = 3;

/// TCP error.
#[derive(Debug, Copy, Clone)]
pub enum TCPError {
    ConnErr,
    ReadErr,
    WriteErr,
    Timeout,
}

pub fn port_number(chain: Chain) -> u16 {
    match chain {
        Chain::Signet | Chain::Testbed => baked::SIGNET_PORT,
        Chain::Mainnet => baked::MAINNET_PORT,
    }
}

pub async fn open_port(chain: Chain) -> bool {
    let port_number = port_number(chain);

    let upnp_config = UpnpConfig {
        address: None,
        port: port_number,
        protocol: PortMappingProtocol::TCP,
        duration: 100_000_000,
        comment: format!("{} {}", baked::PROJECT_TAG, "Transport Layer"),
    };

    for result in add_ports([upnp_config]) {
        if let Ok(_) = result {
            return true;
        }
    }

    false
}

pub async fn connect(ip_address: &str, chain: Chain) -> Result<TcpStream, TCPError> {
    let port_number = port_number(chain);
    let addr = format!("{}:{}", ip_address, port_number);
    let timeout = tokio::time::sleep(Duration::from_millis(3_000));
    let connect = TcpStream::connect(&addr);

    tokio::select! {
        result = connect => {
            match result {
                Ok(stream) => Ok(stream),
                Err(_) => Err(TCPError::ConnErr),
            }
        }
        _ = timeout => Err(TCPError::Timeout),
    }
}

pub async fn connect_nns(
    nns_key: [u8; 32],
    nns_client: &NNSClient,
    chain: Chain,
) -> Result<TcpStream, TCPError> {
    let npub = match nns_key.to_npub() {
        Some(npub) => npub,
        None => return Err(TCPError::ConnErr),
    };

    let ip_address = match nns_client.query_address(&npub).await {
        Some(ip_address) => ip_address,
        None => return Err(TCPError::ConnErr),
    };

    connect(&ip_address, chain).await
}
pub async fn pop(socket: &mut TcpStream, timeout: Option<Duration>) -> Option<TCPPackage> {
    let start = Instant::now();

    // Read package kind.
    let mut package_kind_buffer = [0x00u8; 1];
    let remaining_time = timeout.and_then(|t| t.checked_sub(start.elapsed()));

    read(socket, &mut package_kind_buffer, remaining_time)
        .await
        .ok()?;

    let package_kind = PackageKind::from_bytecode(package_kind_buffer[0])?;

    // Read timestamp.
    let mut timestamp_buffer = [0x00u8; 8];
    let remaining_time = timeout.and_then(|t| t.checked_sub(start.elapsed()));

    read(socket, &mut timestamp_buffer, remaining_time)
        .await
        .ok()?;

    let timestamp = i64::from_be_bytes(timestamp_buffer);

    // Read payload length.
    let mut payload_length_buffer = [0x00u8; 4];
    let remaining_time = timeout.and_then(|t| t.checked_sub(start.elapsed()));
    read(socket, &mut payload_length_buffer, remaining_time)
        .await
        .ok()?;
    let payload_length = u32::from_be_bytes(payload_length_buffer);

    // Read payload.
    let mut payload_buffer = vec![0; payload_length as usize];
    let remaining_time = timeout.and_then(|t| t.checked_sub(start.elapsed()));
    read(socket, &mut payload_buffer, remaining_time)
        .await
        .ok()?;

    Some(TCPPackage::new(package_kind, timestamp, &payload_buffer))
}

pub async fn read(
    socket: &mut TcpStream,
    buffer: &mut [u8],
    timeout: Option<Duration>,
) -> Result<(), TCPError> {
    if let Some(duration) = timeout {
        tokio::select! {

            result = socket.read_exact(buffer) => {
                // Handle the result of read_exact
                match result {
                    Ok(_) => Ok(()),
                    Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Err(TCPError::ConnErr),
                    Err(_) => Err(TCPError::ReadErr),
                }
            }
            _ = tokio::time::sleep(duration) => {
                // Timeout occurred
                Err(TCPError::Timeout)
            }
        }
    } else {
        // No timeout specified, perform the read_exact operation
        match socket.read_exact(buffer).await {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Err(TCPError::ConnErr),
            Err(_) => Err(TCPError::ReadErr),
        }
    }
}

pub async fn write(
    socket: &mut TcpStream,
    payload: &[u8],
    timeout: Option<Duration>,
) -> Result<(), TCPError> {
    if let Some(duration) = timeout {
        tokio::select! {
            result = socket.write_all(payload) => {
                // Handle the result of write_all
                match result {
                    Ok(_) => Ok(()),
                    Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Err(TCPError::ConnErr),
                    Err(_) => Err(TCPError::WriteErr),
                }
            }
            _ = tokio::time::sleep(duration) => {
                // Timeout occurred
                Err(TCPError::Timeout)
            }
        }
    } else {
        // No timeout specified, perform the write_all operation
        match socket.write_all(payload).await {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Err(TCPError::ConnErr),
            Err(_) => Err(TCPError::WriteErr),
        }
    }
}

pub async fn request(
    socket: &SOCKET,
    package: TCPPackage,
    timeout: Option<Duration>,
) -> Result<(TCPPackage, Duration), TCPError> {
    // Lock the socket.
    let mut _socket = socket.lock().await;

    // Start tracking elapsed time.
    let start = Instant::now();
    let timeout_duration = timeout.unwrap_or(Duration::from_millis(3_000)); // Default timeout: 3000 ms

    // Write the request buffer with timeout.
    write(&mut *_socket, &package.serialize(), Some(timeout_duration)).await?;

    let remaining_time = timeout_duration
        .checked_sub(start.elapsed())
        .ok_or(TCPError::Timeout)?;

    // Read response package.
    tokio::select! {
        result = async {
            loop {
                let remaining_time = timeout_duration
        .checked_sub(start.elapsed())
        .ok_or(TCPError::Timeout)?;


                let response_package = match pop(&mut *_socket, Some(remaining_time)).await {
                    Some(package) => package,
                    None => return Err(TCPError::Timeout),
                };

                if response_package.kind() == package.kind() && response_package.timestamp() == package.timestamp() {
                    return Ok((response_package, start.elapsed()));
                }
            }
        } => result, // Pass the loop's result directly
        _ = sleep(remaining_time) => {
            // Timeout branch must return the same type
            Err(TCPError::Timeout)
        }
    }
}

pub async fn connectivity() -> bool {
    match tokio::time::timeout(
        Duration::from_secs(TCP_RESPONSE_TIMEOUT),
        TcpStream::connect("8.8.8.8:53"),
    )
    .await
    {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}
