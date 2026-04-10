use crate::constructive::core_types::target::target::Target;
use crate::constructive::{
    entity::account::root_account::root_account::RootAccount,
    entry::entry_types::liftup::liftup::Liftup, txo::lift::lift::Lift,
};
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
// lift
pub async fn lift_command(
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    v2_lift_enabled: bool,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    utxo_set: &UTXO_SET,
    registery: &REGISTERY,
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
    let current_batch_height_tip: u64 = {
        // 4.1 Lock the sync manager.
        let _sync_manager = sync_manager.lock().await;

        // 4.2 Get the current cube batch height tip.
        _sync_manager.cube_batch_sync_height_tip()
    };

    // 5 Construct the target.
    let target = Target::new(current_batch_height_tip);

    // 6 Construct the Liftup.
    let _liftup = Liftup::new(root_account, target, self_owned_lifts);
}
