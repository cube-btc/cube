use std::time::Duration;

use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::r#move::{
    MoveRequestBody, MoveResponseBody, MoveResponseError,
};
use crate::operative::tasks::engine_session::session_pool::error::exec_move_in_pool_error::ExecMoveInPoolError;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use tokio::time::sleep;

/// Backoff when the session is not ready yet (`SessionInactive` / `SessionBreak`); lock is dropped before sleeping.
const SESSION_SETTLE_MS: u64 = 500;

/// Total `exec_move_in_pool` attempts on session-settle errors.
const MAX_EXEC_ATTEMPTS: u32 = 4;

pub async fn handle_move_request(
    timestamp: i64,
    payload: &[u8],
    session_pool: &SESSION_POOL,
) -> Option<TCPPackage> {
    // 1 Deserialize request body.
    let MoveRequestBody {
        move_entry,
        move_bls_signature,
    } = match MoveRequestBody::deserialize(payload) {
        Some(req) => req,
        None => {
            let body = MoveResponseBody::err(MoveResponseError::DeserializeMoveRequestError);
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(PackageKind::MoveProtocol, timestamp, &bytes));
        }
    };

    // 2 Execute move in session pool with settle retries.
    let mut response: Option<MoveResponseBody> = None;
    for attempt in 1..=MAX_EXEC_ATTEMPTS {
        let attempt_result = {
            let mut _session_pool = session_pool.lock().await;
            _session_pool
                .exec_move_in_pool(&move_entry, move_bls_signature)
                .await
        };

        match attempt_result {
            Ok((entry_id, entry, batch_height, batch_timestamp)) => {
                response = Some(MoveResponseBody::ok(
                    entry_id,
                    batch_height,
                    batch_timestamp,
                    entry,
                ));
                break;
            }
            Err(err) => {
                let retry_after_settle = matches!(
                    err,
                    ExecMoveInPoolError::SessionInactiveError
                        | ExecMoveInPoolError::SessionBreakError
                );

                if retry_after_settle && attempt < MAX_EXEC_ATTEMPTS {
                    sleep(Duration::from_millis(SESSION_SETTLE_MS)).await;
                    continue;
                }

                response = Some(MoveResponseBody::err(MoveResponseError::ExecMoveInPoolError(
                    err,
                )));
                break;
            }
        }
    }

    // 3 Serialize response body.
    let response_bytes = response
        .expect("This can never be None after the loop.")
        .serialize()
        .unwrap_or_default();

    // 4 Construct response package.
    let response_package = TCPPackage::new(PackageKind::MoveProtocol, timestamp, &response_bytes);

    // 5 Return response package.
    Some(response_package)
}
