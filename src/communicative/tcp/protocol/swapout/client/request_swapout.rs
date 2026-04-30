//! Send helper for Swapout TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::swapout::{SwapoutRequestBody, SwapoutResponseBody};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use chrono::Utc;
use std::time::Duration;

const SWAPOUT_REQUEST_TIMEOUT_MS: u64 = 5_000;

pub async fn request_swapout(
    peer: &PEER,
    swapout: &Swapout,
    swapout_bls_signature: [u8; 96],
) -> Result<(SwapoutResponseBody, Duration), RequestError> {
    let request_body = SwapoutRequestBody::new(swapout.clone(), swapout_bls_signature);

    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    let request_package = TCPPackage::new(
        PackageKind::SwapoutProtocol,
        Utc::now().timestamp(),
        &payload,
    );

    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    let timeout = Duration::from_millis(SWAPOUT_REQUEST_TIMEOUT_MS);

    let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
        .await
        .map_err(RequestError::TCPErr)?;

    let response_payload = match response_package.payload_len() {
        0 => return Err(RequestError::EmptyResponse),
        _ => response_package.payload(),
    };

    SwapoutResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
