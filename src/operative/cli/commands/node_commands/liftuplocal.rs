use std::sync::Arc;

use crate::constructive::core_types::target::target::Target;
use crate::constructive::{
    entity::account::root_account::root_account::RootAccount,
    entry::entry_kinds::liftup::liftup::Liftup, txo::lift::lift::Lift,
};
use crate::executive::exec_ctx::exec_ctx::{ExecCtx, EXEC_CTX};
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::params_manager::params_manager::PARAMS_MANAGER;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::key::KeyHolder;
use chrono::Utc;
use colored::Colorize;
use serde_json::to_string_pretty;

// liftuplocal
pub async fn liftup_local_command(
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    v2_lift_enabled: bool,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    utxo_set: &UTXO_SET,
    registery: &REGISTERY,
    graveyard: &GRAVEYARD,
    coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
    state_manager: &STATE_MANAGER,
    privileges_manager: &PRIVILEGES_MANAGER,
    params_manager: &PARAMS_MANAGER,
    archival_manager: Option<ARCHIVAL_MANAGER>,
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
    let root_account = RootAccount::self_root_account_from_registery(key_holder, registery).await;

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

    // 7 Construct the execution timestamp.
    let execution_timestamp = Utc::now().timestamp() as u64;

    // 8 Construct the Liftup.
    let liftup = Liftup::new(root_account, target, self_owned_lifts);

    // 9 Construct exec ctx.
    let exec_ctx: EXEC_CTX = ExecCtx::construct(
        engine_key,
        Arc::clone(sync_manager),
        Arc::clone(utxo_set),
        Arc::clone(registery),
        Arc::clone(graveyard),
        Arc::clone(coin_manager),
        Arc::clone(flame_manager),
        Arc::clone(state_manager),
        Arc::clone(privileges_manager),
        Arc::clone(params_manager),
        archival_manager,
    );

    // 10 Execute the liftup in the exec ctx.
    let execute_liftup_result = {
        let mut _exec_ctx = exec_ctx.lock().await;
        _exec_ctx.execute_liftup(&liftup, execution_timestamp).await
    };

    // 11 Flush the exec ctx.
    {
        let mut _exec_ctx = exec_ctx.lock().await;
        _exec_ctx.flush().await;
    }

    // 12 Drop the exec ctx.
    drop(exec_ctx);

    // 13 Match the execute liftup result.
    match execute_liftup_result {
        Ok(liftup_entry) => {
            println!(
                "{}",
                format!(
                    "Liftup entry successfully executed: {}",
                    to_string_pretty(&liftup_entry.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        Err(error) => {
            println!("{}", format!("Error executing liftup: {:?}", error).red());
        }
    }
}
