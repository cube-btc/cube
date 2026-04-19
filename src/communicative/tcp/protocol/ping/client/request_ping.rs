//! Send helper for ping TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use chrono::Utc;
use std::time::Duration;

/// Sends a ping over the peer's TCP connection; returns round-trip time on success.
pub async fn request_ping(peer: &PEER) -> Result<Duration, RequestError> {
    let payload = [0x00u8];

    let request_package = {
        let kind = PackageKind::Ping;
        let timestamp = Utc::now().timestamp();
        TCPPackage::new(kind, timestamp, &payload)
    };

    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    let timeout = Duration::from_millis(3_000);

    let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
        .await
        .map_err(RequestError::TCPErr)?;

    let response_payload = match response_package.payload_len() {
        0 => return Err(RequestError::EmptyResponse),
        _ => response_package.payload(),
    };

    if response_payload != [0x01u8] {
        return Err(RequestError::InvalidResponse);
    }

    Ok(duration)
}
