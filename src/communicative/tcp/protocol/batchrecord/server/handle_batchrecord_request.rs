use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::batchrecord::{
    BatchRecordRequestBody, BatchRecordResponseBody, BatchRecordResponseError,
};
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;

pub async fn handle_batchrecord_request(
    timestamp: i64,
    payload: &[u8],
    session_pool: &SESSION_POOL,
) -> Option<TCPPackage> {
    // 1 Deserialize the request body.
    let BatchRecordRequestBody { batch_height } = match BatchRecordRequestBody::deserialize(
        payload,
    ) {
        Some(req) => req,
        None => {
            let body = BatchRecordResponseBody::err(
                BatchRecordResponseError::DeserializeBatchRecordRequestError,
            );
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(
                PackageKind::BatchRecordProtocol,
                timestamp,
                &bytes,
            ));
        }
    };

    // 2 Resolve the batch record from the engine archival manager (if configured).
    let response_body = {
        let mut _session_pool = session_pool.lock().await;
        let _exec_ctx = _session_pool.exec_ctx.lock().await;

        match &_exec_ctx.archival_manager {
            None => BatchRecordResponseBody::err(
                BatchRecordResponseError::ArchivalManagerUnavailableError,
            ),
            Some(archival_manager) => {
                let _archival_manager = archival_manager.lock().await;
                let batch_record = _archival_manager.batch_record_by_height(batch_height);
                BatchRecordResponseBody::ok(batch_record)
            }
        }
    };

    // 3 Serialize the response body.
    let response_bytes = response_body.serialize().unwrap_or_default();

    // 4 Construct the response package.
    let response_package =
        TCPPackage::new(PackageKind::BatchRecordProtocol, timestamp, &response_bytes);

    // 5 Return the response package.
    Some(response_package)
}
