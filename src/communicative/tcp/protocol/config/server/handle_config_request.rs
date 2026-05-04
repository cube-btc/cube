use std::time::Duration;

use crate::communicative::tcp::package::{PackageKind, TCPPackage};
use crate::communicative::tcp::protocol::config::{
    ConfigRequestBody, ConfigResponseBody, ConfigResponseError,
};
use crate::operative::tasks::engine_session::session_pool::error::exec_config_in_pool_error::ExecConfigInPoolError;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use tokio::time::sleep;

const SESSION_SETTLE_MS: u64 = 500;
const MAX_EXEC_ATTEMPTS: u32 = 4;

pub async fn handle_config_request(
    timestamp: i64,
    payload: &[u8],
    session_pool: &SESSION_POOL,
) -> Option<TCPPackage> {
    let ConfigRequestBody {
        config,
        config_bls_signature,
    } = match ConfigRequestBody::deserialize(payload) {
        Some(req) => req,
        None => {
            let body = ConfigResponseBody::err(ConfigResponseError::DeserializeConfigRequestError);
            let bytes = body.serialize().unwrap_or_default();
            return Some(TCPPackage::new(
                PackageKind::ConfigProtocol,
                timestamp,
                &bytes,
            ));
        }
    };

    let mut response: Option<ConfigResponseBody> = None;
    for attempt in 1..=MAX_EXEC_ATTEMPTS {
        let attempt_result = {
            let mut _session_pool = session_pool.lock().await;
            _session_pool
                .exec_config_in_pool(&config, config_bls_signature)
                .await
        };

        match attempt_result {
            Ok((entry_id, entry, batch_height, batch_timestamp)) => {
                response = Some(ConfigResponseBody::ok(
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
                    ExecConfigInPoolError::SessionInactiveError
                        | ExecConfigInPoolError::SessionBreakError
                );

                if retry_after_settle && attempt < MAX_EXEC_ATTEMPTS {
                    sleep(Duration::from_millis(SESSION_SETTLE_MS)).await;
                    continue;
                }

                response = Some(ConfigResponseBody::err(
                    ConfigResponseError::ExecConfigInPoolError(err),
                ));
                break;
            }
        }
    }

    let response_bytes = response
        .expect("This can never be None after the loop.")
        .serialize()
        .unwrap_or_default();

    let response_package = TCPPackage::new(PackageKind::ConfigProtocol, timestamp, &response_bytes);

    Some(response_package)
}
