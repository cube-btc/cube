use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use serde_json::{json, to_string_pretty};

/// Prints sync tips and payload tip txid as JSON.
pub async fn tip_command(sync_manager: &SYNC_MANAGER) {
    let (bitcoin_height, batch_height, txid) = {
        let sm = sync_manager.lock().await;
        let bitcoin_height = sm.bitcoin_sync_height_tip();
        let batch_height = sm.cube_batch_sync_height_tip();
        let txid = sm
            .payload_tip()
            .location()
            .map(|(outpoint, _)| outpoint.txid.to_string());
        (bitcoin_height, batch_height, txid)
    };

    let body = json!({
        "bitcoin_height": bitcoin_height,
        "batch_height": batch_height,
        "txid": txid,
    });

    println!(
        "{}",
        to_string_pretty(&body).expect("serde_json::Value should serialize")
    );
}
