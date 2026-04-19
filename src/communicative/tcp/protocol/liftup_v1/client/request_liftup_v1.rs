//! Send helper for Liftup v1 TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::liftup_v1::{LiftupV1RequestBody, LiftupV1ResponseBody};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use chrono::Utc;
use std::time::Duration;

/// Timeout for Liftup v1 requests.
const LIFTUP_V1_REQUEST_TIMEOUT_MS: u64 = 5_000;

/// Sends a Liftup v1 request over the peer's TCP connection.
pub async fn request_liftup_v1(
    peer: &PEER,
    liftup: &Liftup,
    liftup_bls_signature: [u8; 96],
) -> Result<(LiftupV1ResponseBody, Duration), RequestError> {
    // 1 Construct the request body.
    let request_body = LiftupV1RequestBody::new(liftup.clone(), liftup_bls_signature);

    // 2 Serialize the request body.
    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    // 3 Construct the request package.
    let request_package = TCPPackage::new(
        PackageKind::LiftupV1Protocol,
        Utc::now().timestamp(),
        &payload,
    );

    // 4 Send the request package.
    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    // 5 Set the timeout.
    let timeout = Duration::from_millis(LIFTUP_V1_REQUEST_TIMEOUT_MS);

    // 6 Send the request package and get the response package.
    let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
        .await
        .map_err(RequestError::TCPErr)?;

    // 7 Deserialize the response payload.
    let response_payload = match response_package.payload_len() {
        0 => return Err(RequestError::EmptyResponse),
        _ => response_package.payload(),
    };

    // 8 Return the response body.
    LiftupV1ResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
