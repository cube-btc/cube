use super::package::{PackageKind, TCPPackage};
use super::tcp::{self, TCPError};
use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::constructive::entry::entries::liftup::liftup::Liftup;
use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;

// A type alias for bytes.
type Bytes = Vec<u8>;

#[async_trait]
pub trait TCPClient {
    async fn ping(&self) -> Result<Duration, RequestError>;
    async fn request_liftup_v1(&self, liftup: &Liftup) -> Result<(Bytes, Duration), RequestError>;
}

#[derive(Copy, Clone)]
pub enum RequestError {
    TCPErr(TCPError),
    InvalidRequest,
    InvalidResponse,
    EmptyResponse,
    ErrorResponse,
    BincodeSerializationError,
}

#[async_trait]
impl TCPClient for PEER {
    /// Pinging.
    async fn ping(&self) -> Result<Duration, RequestError> {
        // 1 Build the payload.
        let payload = [0x00u8];

        // 2 Build the request package.
        let request_package = {
            let kind = PackageKind::Ping;
            let timestamp = Utc::now().timestamp();
            TCPPackage::new(kind, timestamp, &payload)
        };

        // 3 Get the socket.
        let socket: SOCKET = self
            .socket()
            .await
            .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

        // 4 Set the timeout.
        let timeout = Duration::from_millis(3_000);

        // 5 Send the request package.
        let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
            .await
            .map_err(|err| RequestError::TCPErr(err))?;

        // 6 Get the response payload.
        let response_payload = match response_package.payload_len() {
            0 => return Err(RequestError::EmptyResponse),
            _ => response_package.payload(),
        };

        // 7 Validate the response payload.
        if response_payload != [0x01u8] {
            return Err(RequestError::InvalidResponse);
        }

        // 8 Return the duration.
        Ok(duration)
    }

    /// Non-interactive Liftup V1 request.
    ///
    /// Used to request a liftup v1 from the Engine.
    async fn request_liftup_v1(&self, liftup: &Liftup) -> Result<(Bytes, Duration), RequestError> {
        // 1 Serialize the liftup.
        let payload = bincode::serde::encode_to_vec(&liftup, bincode::config::standard())
            .map_err(|_| RequestError::BincodeSerializationError)?;

        // 2 Build the request package.
        let request_package = TCPPackage::new(
            PackageKind::LiftupV1Protocol,
            Utc::now().timestamp(),
            &payload,
        );

        // 3 Get the socket.
        let socket: SOCKET = self
            .socket()
            .await
            .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

        // 4 Set the timeout.
        let timeout = Duration::from_millis(3_000);

        // 5 Send the request package.
        let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
            .await
            .map_err(|err| RequestError::TCPErr(err))?;

        // 6 Get the response payload.
        let response_payload = match response_package.payload_len() {
            0 => return Err(RequestError::EmptyResponse),
            _ => response_package.payload(),
        };

        // 7 Return the response payload and duration.
        Ok((response_payload, duration))
    }
}
