use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::transmutative::key::KeyHolder;
use serde_json::to_string_pretty;

/// Constructs and prints the self root account JSON.
pub async fn rootaccount_command(key_holder: &KeyHolder, registery: &REGISTERY) {
    let root_account = RootAccount::self_root_account_from_registery(key_holder, registery).await;
    println!(
        "{}",
        to_string_pretty(&root_account.json()).expect("serde_json::Value should serialize")
    );
}
