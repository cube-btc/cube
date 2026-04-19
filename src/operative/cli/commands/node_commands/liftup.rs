use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{LiftupV1ResponseBody, TCPClient};
use crate::constructive::core_types::target::target::Target;
use crate::constructive::{
    entity::account::root_account::root_account::RootAccount,
    entry::entry_kinds::liftup::liftup::Liftup, txo::lift::lift::Lift,
};
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use serde_json::to_string_pretty;

// liftup
pub async fn liftup_command(
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    v2_lift_enabled: bool,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    utxo_set: &UTXO_SET,
    registery: &REGISTERY,
    engine_peer: &PEER,
) {
    // 1 Scan the UTXO set and collect the self owned lifts.
    let self_owned_lifts: Vec<Lift> = {
        let _utxo_set = utxo_set.lock().await;
        _utxo_set.scan_and_return_self_owned_lifts(&engine_key, &self_account_key, v2_lift_enabled)
    };

    // 2 If there are no self owned lifts, print an error message.
    if self_owned_lifts.is_empty() {
        println!("{}", "No lift UTXOs found to liftup.".red());
        return;
    }

    // 3 Construct the Root Account.
    let root_account = RootAccount::self_root_account(key_holder, registery).await;

    // 4 Get the current cube batch height tip from the sync manager.
    let batch_height_tip: u64 = {
        // 4.1 Lock the sync manager.
        let _sync_manager = sync_manager.lock().await;

        // 4.2 Get the current cube batch height tip.
        _sync_manager.cube_batch_sync_height_tip()
    };

    // 5 The current execution batch height is the batch height tip plus one.
    let current_execution_batch_height = batch_height_tip + 1;

    // 6 Construct the target.
    let target = Target::new(current_execution_batch_height);

    // 7 Construct the Liftup.
    let liftup = Liftup::new(root_account, target, self_owned_lifts);

    // 8 Get the BLS signature of the Liftup.
    let liftup_bls_signature: [u8; 96] = {
        match liftup.bls_sign(key_holder) {
            Ok(signature) => signature,
            Err(error) => {
                println!("{}", format!("Error BLS signing liftup: {:?}", error).red());
                return;
            }
        }
    };

    // 9 Request the liftup to the peer.
    let (liftup_v1_response_body, duration) = match engine_peer
        .request_liftup_v1(&liftup, liftup_bls_signature)
        .await
    {
        Ok((liftup_v1_response_body, duration)) => (liftup_v1_response_body, duration),
        Err(error) => {
            println!("{}", format!("Error requesting liftup: {:?}", error).red());
            return;
        }
    };

    // 10 Match the execute liftup result (wire enum, not `Result`).
    match liftup_v1_response_body {
        LiftupV1ResponseBody::Ok(success_body) => {
            println!(
                "{}",
                format!(
                    "Liftup entry successfully executed ({} ms): {}",
                    duration.as_millis(),
                    to_string_pretty(&success_body.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        LiftupV1ResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error executing liftup: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
