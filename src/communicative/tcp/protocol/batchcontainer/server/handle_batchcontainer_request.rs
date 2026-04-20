use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::batchcontainer::{
    BatchContainerRequestBody, BatchContainerResponseBody, BatchContainerResponseError,
};
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;

pub async fn handle_batchcontainer_request(
    timestamp: i64,
    payload: &[u8],
    archival_manager: &Option<ARCHIVAL_MANAGER>,
) -> Option<TCPPackage> {
    // 1 Deserialize the request body.
    let BatchContainerRequestBody { batch_height } = match BatchContainerRequestBody::deserialize(
        payload,
    ) {
        Some(req) => req,
        None => {
            let body = BatchContainerResponseBody::err(
                BatchContainerResponseError::DeserializeBatchContainerRequestError,
            );
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(
                PackageKind::BatchContainerProtocol,
                timestamp,
                &bytes,
            ));
        }
    };

    // 2 Resolve the batch container from the archival manager (if configured).
    let response_body = match archival_manager {
        None => {
            BatchContainerResponseBody::err(BatchContainerResponseError::ArchivalManagerUnavailableError)
        }
        Some(archival_manager) => {
            let _archival_manager = archival_manager.lock().await;
            let batch_container = _archival_manager
                .batch_record_by_height(batch_height)
                .map(|batch_record| batch_record.batch_container.clone());
            BatchContainerResponseBody::ok(batch_container)
        }
    };

    // 3 Serialize the response body.
    let response_bytes = response_body.serialize().unwrap_or_default();

    // 4 Construct the response package.
    let response_package =
        TCPPackage::new(PackageKind::BatchContainerProtocol, timestamp, &response_bytes);

    // 5 Return the response package.
    Some(response_package)
}
