use super::package::{PackageKind, TCPPackage};
use super::tcp::{self, TCPError};
use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;

#[async_trait]
pub trait TCPClient {
    async fn ping(&self) -> Result<Duration, RequestError>;
}

#[derive(Copy, Clone)]
pub enum RequestError {
    TCPErr(TCPError),
    InvalidRequest,
    InvalidResponse,
    EmptyResponse,
    ErrorResponse,
}

#[async_trait]
impl TCPClient for PEER {
    /// Pinging.
    async fn ping(&self) -> Result<Duration, RequestError> {
        let payload = [0x00u8];

        // Build request package.
        let request_package = {
            let kind = PackageKind::Ping;
            let timestamp = Utc::now().timestamp();
            TCPPackage::new(kind, timestamp, &payload)
        };

        // Return the TCP socket.
        let socket: SOCKET = self
            .socket()
            .await
            .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

        // Wait for the 'pong' for 3 seconds.
        let timeout = Duration::from_millis(3_000);

        let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
            .await
            .map_err(|err| RequestError::TCPErr(err))?;

        let response_payload = match response_package.payload_len() {
            0 => return Err(RequestError::EmptyResponse),
            _ => response_package.payload(),
        };

        // Expected response: 0x01 for pong.
        if response_payload != [0x01u8] {
            return Err(RequestError::InvalidResponse);
        }

        Ok(duration)
    }
}
