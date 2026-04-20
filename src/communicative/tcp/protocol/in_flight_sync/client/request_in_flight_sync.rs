//! Send helper for In-flight sync TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::in_flight_sync::{
    InFlightSyncRequestBody, InFlightSyncResponseBody,
};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use chrono::Utc;
use std::time::Duration;

/// Timeout for in-flight sync requests.
const IN_FLIGHT_SYNC_REQUEST_TIMEOUT_MS: u64 = 5_000;

/// Sends an in-flight sync request over the peer's TCP connection.
pub async fn request_in_flight_sync(
    peer: &PEER,
    cube_batch_sync_height_tip: u64,
) -> Result<(InFlightSyncResponseBody, Duration), RequestError> {
    // 1 Construct the request body.
    let request_body = InFlightSyncRequestBody::new(cube_batch_sync_height_tip);

    // 2 Serialize the request body.
    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    // 3 Construct the request package.
    let request_package = TCPPackage::new(
        PackageKind::InFlightSyncProtocol,
        Utc::now().timestamp(),
        &payload,
    );

    // 4 Send the request package.
    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    // 5 Set the timeout.
    let timeout = Duration::from_millis(IN_FLIGHT_SYNC_REQUEST_TIMEOUT_MS);

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
    InFlightSyncResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
