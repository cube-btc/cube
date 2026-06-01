use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::registry::registry::REGISTRY;
use crate::transmutative::key::KeyHolder;
use serde_json::to_string_pretty;

/// Constructs and prints the self root account JSON.
pub async fn rootaccount_command(key_holder: &KeyHolder, registry: &REGISTRY) {
    let root_account = RootAccount::self_root_account_from_registry(key_holder, registry).await;
    println!(
        "{}",
        to_string_pretty(&root_account.json()).expect("serde_json::Value should serialize")
    );
}
