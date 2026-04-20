use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use serde_json::to_string_pretty;

/// Prints the flame manager JSON.
pub async fn flamemanager_command(flame_manager: &FLAME_MANAGER) {
    let flame_manager_json = {
        let _flame_manager = flame_manager.lock().await;
        _flame_manager.json()
    };

    println!(
        "{}",
        to_string_pretty(&flame_manager_json).expect("serde_json::Value should serialize")
    );
}
