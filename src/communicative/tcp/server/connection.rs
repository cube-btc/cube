use super::server::{IDLE_CLIENT_TIMEOUT, PAYLOAD_READ_TIMEOUT, PAYLOAD_WRITE_TIMEOUT};
use crate::communicative::peer::peer::SOCKET;
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::tcp;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::operative::run_args::operating_kind::OperatingKind;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use crate::transmutative::key::KeyHolder;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

pub async fn handle_socket(
    socket: &SOCKET,
    alive: Option<&Arc<Mutex<bool>>>,
    operating_kind: OperatingKind,
    _keys: &KeyHolder,
    session_pool: &SESSION_POOL,
    archival_manager: &Option<ARCHIVAL_MANAGER>,
) {
    loop {
        let package = {
            let mut _socket = socket.lock().await;

            let mut package_kind_buffer = [0; 1];
            match tcp::read(
                &mut *_socket,
                &mut package_kind_buffer,
                Some(IDLE_CLIENT_TIMEOUT),
            )
            .await
            {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break,
                Err(tcp::TCPError::Timeout) => break,
                Err(_) => continue,
            }
            let package_kind = match PackageKind::from_bytecode(package_kind_buffer[0]) {
                Some(kind) => kind,
                None => continue,
            };

            let start = Instant::now();
            let timeout_duration = PAYLOAD_READ_TIMEOUT;

            let mut timestamp_buffer = [0; 8];
            match tcp::read(&mut *_socket, &mut timestamp_buffer, Some(timeout_duration)).await {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break,
                Err(tcp::TCPError::Timeout) => continue,
                Err(_) => continue,
            }
            let timestamp = i64::from_be_bytes(timestamp_buffer);

            let remaining_time = match timeout_duration.checked_sub(start.elapsed()) {
                Some(duration) => duration,
                None => continue,
            };

            let mut payload_len_buffer = [0; 4];
            match tcp::read(&mut *_socket, &mut payload_len_buffer, Some(remaining_time)).await {
                Ok(_) => (),
                Err(tcp::TCPError::ConnErr) => break,
                Err(tcp::TCPError::Timeout) => continue,
                Err(_) => continue,
            }
            let payload_len = u32::from_be_bytes(payload_len_buffer) as usize;

            let remaining_time = match timeout_duration.checked_sub(start.elapsed()) {
                Some(duration) => duration,
                None => continue,
            };

            let mut payload_bufer = vec![0x00u8; u32::from_be_bytes(payload_len_buffer) as usize];
            match payload_len {
                0 => continue,
                _ => {
                    match tcp::read(&mut *_socket, &mut payload_bufer, Some(remaining_time)).await {
                        Ok(_) => (),
                        Err(tcp::TCPError::ConnErr) => break,
                        Err(tcp::TCPError::Timeout) => continue,
                        Err(_) => continue,
                    }
                }
            }

            TCPPackage::new(package_kind, timestamp, &payload_bufer)
        };

        let session_pool = Arc::clone(session_pool);
        let archival_manager = archival_manager.clone();
        handle_package(
            package,
            socket,
            operating_kind,
            _keys,
            &session_pool,
            &archival_manager,
        )
        .await;
    }

    if let Some(alive) = alive {
        let mut _alive = alive.lock().await;
        *_alive = false;
    }
}

pub async fn handle_package(
    package: TCPPackage,
    socket: &SOCKET,
    operating_kind: OperatingKind,
    _keys: &KeyHolder,
    session_pool: &SESSION_POOL,
    archival_manager: &Option<ARCHIVAL_MANAGER>,
) {
    let response_package_ = {
        match operating_kind {
            OperatingKind::Engine => match package.kind() {
                PackageKind::Ping => {
                    crate::communicative::tcp::protocol::ping::server::handle_ping_request(
                        package.timestamp(),
                        &package.payload(),
                    )
                    .await
                }
                PackageKind::LiftupV1Protocol => {
                    let session_pool = Arc::clone(session_pool);
                    crate::communicative::tcp::protocol::liftup_v1::server::handle_liftup_v1_request(
                        package.timestamp(),
                        &package.payload(),
                        &session_pool,
                    )
                    .await
                }
                PackageKind::MoveProtocol => {
                    let session_pool = Arc::clone(session_pool);
                    crate::communicative::tcp::protocol::r#move::server::handle_move_request(
                        package.timestamp(),
                        &package.payload(),
                        &session_pool,
                    )
                    .await
                }
                PackageKind::SwapoutProtocol => {
                    let session_pool = Arc::clone(session_pool);
                    crate::communicative::tcp::protocol::swapout::server::handle_swapout_request(
                        package.timestamp(),
                        &package.payload(),
                        &session_pool,
                    )
                    .await
                }
                PackageKind::BatchRecordProtocol => {
                    let archival_manager = archival_manager.clone();
                    crate::communicative::tcp::protocol::batchrecord::server::handle_batchrecord_request(
                        package.timestamp(),
                        &package.payload(),
                        &archival_manager,
                    )
                    .await
                }
                PackageKind::InFlightSyncProtocol => {
                    let session_pool = Arc::clone(session_pool);
                    crate::communicative::tcp::protocol::in_flight_sync::server::handle_in_flight_sync_request::handle_in_flight_sync_request(
                        package.timestamp(),
                        &package.payload(),
                        &session_pool,
                    )
                    .await
                }
                PackageKind::BatchContainerProtocol => {
                    let archival_manager = archival_manager.clone();
                    crate::communicative::tcp::protocol::batchcontainer::server::handle_batchcontainer_request(
                        package.timestamp(),
                        &package.payload(),
                        &archival_manager,
                    )
                    .await
                }
                PackageKind::BatchContainerByPrevOutpointProtocol => {
                    let archival_manager = archival_manager.clone();
                    crate::communicative::tcp::protocol::batchcontainer_by_prevoutpoint::server::handle_batchcontainer_by_prevoutpoint_request(
                        package.timestamp(),
                        &package.payload(),
                        &archival_manager,
                    )
                    .await
                }
            },
            OperatingKind::Node => return,
        }
    };

    let response_package = match response_package_ {
        Some(package) => package,
        None => TCPPackage::new(package.kind(), package.timestamp(), &[]),
    };

    let _ = response_package
        .deliver(socket, Some(PAYLOAD_WRITE_TIMEOUT))
        .await;
}
