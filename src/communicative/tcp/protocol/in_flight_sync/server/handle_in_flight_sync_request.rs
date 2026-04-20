use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::in_flight_sync::{
    InFlightSyncRequestBody, InFlightSyncResponseBody, InFlightSyncResponseError,
};
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;

pub async fn handle_in_flight_sync_request(
    timestamp: i64,
    payload: &[u8],
    session_pool: &SESSION_POOL,
) -> Option<TCPPackage> {
    // 1 Deserialize the request body.
    let InFlightSyncRequestBody {
        cube_batch_sync_height_tip,
    } = match InFlightSyncRequestBody::deserialize(payload) {
        Some(req) => req,
        None => {
            let body = InFlightSyncResponseBody::err(
                InFlightSyncResponseError::DeserializeInFlightSyncRequestError,
            );
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(
                PackageKind::InFlightSyncProtocol,
                timestamp,
                &bytes,
            ));
        }
    };

    // 2 Resolve the next batch container from the engine archival manager.
    let response_body = {
        let mut _session_pool = session_pool.lock().await;
        let _exec_ctx = _session_pool.exec_ctx.lock().await;

        let engine_tip = {
            let _sync_manager = _exec_ctx.sync_manager.lock().await;
            _sync_manager.cube_batch_sync_height_tip()
        };

        if cube_batch_sync_height_tip >= engine_tip {
            InFlightSyncResponseBody::fully_synced()
        } else {
            let next_batch_height = cube_batch_sync_height_tip + 1;

            match &_exec_ctx.archival_manager {
                None => InFlightSyncResponseBody::err(
                    InFlightSyncResponseError::ArchivalManagerUnavailableError,
                ),
                Some(archival_manager) => {
                    let _archival_manager = archival_manager.lock().await;
                    let batch_container = _archival_manager
                        .batch_record_by_height(next_batch_height)
                        .map(|batch_record| batch_record.batch_container.clone());

                    match batch_container {
                        Some(batch_container) => {
                            InFlightSyncResponseBody::batch_download(batch_container)
                        }
                        None => InFlightSyncResponseBody::err(
                            InFlightSyncResponseError::BatchContainerNotFoundError,
                        ),
                    }
                }
            }
        }
    };

    // 3 Serialize the response body.
    let response_bytes = response_body.serialize().unwrap_or_default();

    // 4 Construct the response package.
    let response_package =
        TCPPackage::new(PackageKind::InFlightSyncProtocol, timestamp, &response_bytes);

    // 5 Return the response package.
    Some(response_package)
}
