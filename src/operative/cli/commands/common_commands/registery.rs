use crate::inscriptive::registery::registery::REGISTERY;
use serde_json::to_string_pretty;

/// Prints the registery manager JSON.
pub async fn registery_command(registery: &REGISTERY) {
    let registery_json = {
        let _registery = registery.lock().await;
        _registery.json()
    };

    println!(
        "{}",
        to_string_pretty(&registery_json).expect("serde_json::Value should serialize")
    );
}
