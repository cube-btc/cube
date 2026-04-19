use std::time::Duration;

use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::liftup_v1::{
    LiftupV1RequestBody, LiftupV1ResponseBody, LiftupV1ResponseError,
};
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_in_pool_error::ExecLiftupInPoolError;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use tokio::time::sleep;

/// Backoff when the session is not ready yet (`SessionInactive` / `SessionBreak`); lock is dropped before sleeping.
const SESSION_SETTLE_MS: u64 = 500;

/// Total `exec_liftup_in_pool` attempts on session-settle errors (initial try plus waits between retries).
const MAX_EXEC_ATTEMPTS: u32 = 4;

pub async fn handle_liftup_v1_request(
    timestamp: i64,
    payload: &[u8],
    session_pool: &SESSION_POOL,
) -> Option<TCPPackage> {
    // 1 Deserialize the request body.
    let LiftupV1RequestBody {
        liftup,
        liftup_bls_signature,
    } = match LiftupV1RequestBody::deserialize(payload) {
        Some(req) => req,
        None => {
            let body =
                LiftupV1ResponseBody::err(LiftupV1ResponseError::DeserializeLiftupV1RequestError);
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(
                PackageKind::LiftupV1Protocol,
                timestamp,
                &bytes,
            ));
        }
    };

    // 2 Execute the liftup in the session pool; up to 4 attempts with 500ms between tries on settle errors.
    let mut response: Option<LiftupV1ResponseBody> = None;
    for attempt in 1..=MAX_EXEC_ATTEMPTS {
        let attempt_result = {
            let mut _session_pool = session_pool.lock().await;
            _session_pool
                .exec_liftup_in_pool(&liftup, liftup_bls_signature)
                .await
        };

        match attempt_result {
            Ok((entry_id, entry, batch_height, batch_timestamp)) => {
                response = Some(LiftupV1ResponseBody::ok(
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
                    ExecLiftupInPoolError::SessionInactiveError
                        | ExecLiftupInPoolError::SessionBreakError
                );

                if retry_after_settle && attempt < MAX_EXEC_ATTEMPTS {
                    sleep(Duration::from_millis(SESSION_SETTLE_MS)).await;
                    continue;
                }

                response = Some(LiftupV1ResponseBody::err(
                    LiftupV1ResponseError::ExecLiftupInPoolError(err),
                ));
                break;
            }
        }
    }

    // 3 Serialize the response body.
    let response_bytes = response
        .expect("This can never be None after the loop.")
        .serialize()
        .unwrap_or_default();

    // 4 Construct the response package.
    let response_package =
        TCPPackage::new(PackageKind::LiftupV1Protocol, timestamp, &response_bytes);

    // 5 Return the response package.
    Some(response_package)
}
