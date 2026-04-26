use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::TCPClient;
use crate::communicative::tcp::protocol::in_flight_sync::InFlightSyncResponseBody;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use std::sync::Arc;
use std::time::Duration;

/// Node background loop to fetch in-flight Cube batches from the Engine one-by-one.
pub async fn in_flight_batch_sync_background_task(
    engine_conn: &PEER,
    sync_manager: &SYNC_MANAGER,
    engine_key: [u8; 32],
    utxo_set: &UTXO_SET,
    registery: &REGISTERY,
    graveyard: &GRAVEYARD,
    coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
    state_manager: &STATE_MANAGER,
    privileges_manager: &PRIVILEGES_MANAGER,
    archival_manager: &Option<ARCHIVAL_MANAGER>,
) {
    loop {
        let current_cube_batch_sync_height_tip = {
            let _sync_manager = sync_manager.lock().await;
            _sync_manager.cube_batch_sync_height_tip()
        };

        let in_flight_sync_response = match engine_conn
            .request_in_flight_sync(current_cube_batch_sync_height_tip)
            .await
        {
            Ok((response_body, _)) => response_body,
            Err(error) => {
                eprintln!("In-flight sync request failed: {:?}. Retrying in 5s...", error);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        match in_flight_sync_response {
            InFlightSyncResponseBody::FullySynced => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            InFlightSyncResponseBody::BatchDownload(batch_container) => {
                let exec_ctx = ExecCtx::construct(
                    engine_key,
                    Arc::clone(sync_manager),
                    Arc::clone(utxo_set),
                    Arc::clone(registery),
                    Arc::clone(graveyard),
                    Arc::clone(coin_manager),
                    Arc::clone(flame_manager),
                    Arc::clone(state_manager),
                    Arc::clone(privileges_manager),
                    archival_manager.clone(),
                );

                let execute_batch_result = {
                    let mut _exec_ctx = exec_ctx.lock().await;
                    _exec_ctx.execute_batch(&batch_container).await
                };

                match execute_batch_result {
                    Ok(batch_record) => {
                        println!(
                            "In-flight sync applied batch #{}.",
                            batch_record.batch_height
                        );
                    }
                    Err(error) => {
                        eprintln!(
                            "In-flight sync failed to execute batch #{}: {:?}. Retrying in 5s...",
                            batch_container.batch_height(),
                            error
                        );
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
            InFlightSyncResponseBody::Err(error) => {
                eprintln!(
                    "In-flight sync response error: {:?}. Retrying in 5s...",
                    error
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}
