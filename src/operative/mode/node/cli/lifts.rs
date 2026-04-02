use crate::constructive::txo::lift::lift::Lift;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use serde_json::{to_string_pretty, Value};

/// Whether MuSig2-based interactive lifts are enabled. Set to false for now since it's not supported yet.
const V2_INTERACTIVE_ENABLED: bool = false;

// lifts
pub async fn lifts_command(engine_key: [u8; 32], self_account_key: [u8; 32], utxo_set: &UTXO_SET) {
    // 1 Scan the UTXO set and collect the self owned lifts.
    let self_owned_lifts: Vec<Lift> = {
        let _utxo_set = utxo_set.lock().await;
        _utxo_set.scan_and_return_self_owned_lifts(
            &engine_key,
            &self_account_key,
            V2_INTERACTIVE_ENABLED,
        )
    };

    // 2 Print the lifts as one JSON array.
    let lifts_json: Vec<Value> = self_owned_lifts.into_iter().map(|l| l.json()).collect();
    println!(
        "{}",
        to_string_pretty(&Value::Array(lifts_json)).expect("serde_json::Value should serialize")
    );
}
