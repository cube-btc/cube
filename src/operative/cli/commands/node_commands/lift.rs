use crate::constructive::{
    entity::account::root_account::root_account::RootAccount,
    entry::entries::liftup::liftup::Liftup, txo::lift::lift::Lift,
};
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;

// lift
pub async fn lift_command(
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    v2_lift_enabled: bool,
    key_holder: &KeyHolder,
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

    // 4 Construct the Liftup.
    let _liftup = Liftup::new(root_account, self_owned_lifts);
}
