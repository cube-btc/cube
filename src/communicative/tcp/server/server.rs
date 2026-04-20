use super::connection::handle_socket;
use super::super::tcp::port_number;
use crate::operative::run_args::{chain::Chain, operating_kind::OperatingKind};
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

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
    keys: Arc<KeyHolder>,
    session_pool: &SESSION_POOL,
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
        OperatingKind::Engine => loop {
            let (socket_, _) = match listener.accept().await {
                Ok(conn) => (conn.0, conn.1),
                Err(_) => continue,
            };

            let socket = Arc::new(tokio::sync::Mutex::new(socket_));
            let keys = Arc::clone(&keys);
            let session_pool = Arc::clone(session_pool);

            tokio::spawn(async move {
                handle_socket(&socket, None, operating_kind, &keys, &session_pool).await;
            });
        },
        OperatingKind::Node => return,
    }
}
