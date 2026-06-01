use crate::inscriptive::registry::registry::REGISTRY;
use serde_json::to_string_pretty;

/// Prints the registry manager JSON.
pub async fn registry_command(registry: &REGISTRY) {
    let registry_json = {
        let _registry = registry.lock().await;
        _registry.json()
    };

    println!(
        "{}",
        to_string_pretty(&registry_json).expect("serde_json::Value should serialize")
    );
}
