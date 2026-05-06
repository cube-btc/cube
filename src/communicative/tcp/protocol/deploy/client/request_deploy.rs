//! Send helper for Deploy TCP requests.

use crate::communicative::peer::peer::{PeerConnection, PEER, SOCKET};
use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::deploy::{DeployRequestBody, DeployResponseBody};
use crate::communicative::tcp::request_error::RequestError;
use crate::communicative::tcp::tcp::{self, TCPError};
use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use chrono::Utc;
use std::time::Duration;

const DEPLOY_REQUEST_TIMEOUT_MS: u64 = 5_000;

pub async fn request_deploy(
    peer: &PEER,
    deploy: &Deploy,
    deploy_bls_signature: [u8; 96],
) -> Result<(DeployResponseBody, Duration), RequestError> {
    let request_body = DeployRequestBody::new(deploy.clone(), deploy_bls_signature);

    let payload = request_body
        .serialize()
        .ok_or(RequestError::RequestSerializationError)?;

    let request_package = TCPPackage::new(
        PackageKind::DeployProtocol,
        Utc::now().timestamp(),
        &payload,
    );

    let socket: SOCKET = peer
        .socket()
        .await
        .ok_or(RequestError::TCPErr(TCPError::ConnErr))?;

    let timeout = Duration::from_millis(DEPLOY_REQUEST_TIMEOUT_MS);

    let (response_package, duration) = tcp::request(&socket, request_package, Some(timeout))
        .await
        .map_err(RequestError::TCPErr)?;

    let response_payload = match response_package.payload_len() {
        0 => return Err(RequestError::EmptyResponse),
        _ => response_package.payload(),
    };

    DeployResponseBody::deserialize(&response_payload)
        .ok_or(RequestError::ResponseDeserializationError)
        .map(|r| (r, duration))
}
