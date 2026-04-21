//! Send helper for Move TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::r#move::{MoveRequestBody, MoveResponseBody};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use chrono::Utc;
use std::time::Duration;

/// Timeout for Move requests.
const MOVE_REQUEST_TIMEOUT_MS: u64 = 5_000;

/// Sends a Move request over the peer's TCP connection.
pub async fn request_move(
    peer: &PEER,
    move_entry: &Move,
    move_bls_signature: [u8; 96],
) -> Result<(MoveResponseBody, Duration), RequestError> {
    // 1 Construct the request body.
    let request_body = MoveRequestBody::new(move_entry.clone(), move_bls_signature);

    // 2 Serialize the request body.
    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    // 3 Construct the request package.
    let request_package = TCPPackage::new(
        PackageKind::MoveProtocol,
        Utc::now().timestamp(),
        &payload,
    );

    // 4 Send the request package.
    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    // 5 Set timeout.
    let timeout = Duration::from_millis(MOVE_REQUEST_TIMEOUT_MS);

    // 6 Send request and receive response package.
    let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
        .await
        .map_err(RequestError::TCPErr)?;

    // 7 Deserialize response payload.
    let response_payload = match response_package.payload_len() {
        0 => return Err(RequestError::EmptyResponse),
        _ => response_package.payload(),
    };

    // 8 Return response body.
    MoveResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
