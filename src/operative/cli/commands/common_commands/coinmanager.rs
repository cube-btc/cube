use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use serde_json::to_string_pretty;

/// Prints the coin manager JSON.
pub async fn coinmanager_command(coin_manager: &COIN_MANAGER) {
    let coin_manager_json = {
        let _coin_manager = coin_manager.lock().await;
        _coin_manager.json()
    };

    println!(
        "{}",
        to_string_pretty(&coin_manager_json).expect("serde_json::Value should serialize")
    );
}
