//! Send helper for Batch container TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::batchcontainer::{
    BatchContainerRequestBody, BatchContainerResponseBody,
};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use chrono::Utc;
use std::time::Duration;

/// Timeout for Batch container requests.
const BATCHCONTAINER_REQUEST_TIMEOUT_MS: u64 = 5_000;

/// Sends a Batch container request over the peer's TCP connection.
pub async fn request_batchcontainer(
    peer: &PEER,
    batch_height: u64,
) -> Result<(BatchContainerResponseBody, Duration), RequestError> {
    // 1 Construct the request body.
    let request_body = BatchContainerRequestBody::new(batch_height);

    // 2 Serialize the request body.
    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    // 3 Construct the request package.
    let request_package = TCPPackage::new(
        PackageKind::BatchContainerProtocol,
        Utc::now().timestamp(),
        &payload,
    );

    // 4 Send the request package.
    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    // 5 Set the timeout.
    let timeout = Duration::from_millis(BATCHCONTAINER_REQUEST_TIMEOUT_MS);

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
    BatchContainerResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
