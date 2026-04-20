use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use serde_json::to_string_pretty;

/// Prints the graveyard manager JSON.
pub async fn graveyard_command(graveyard: &GRAVEYARD) {
    let graveyard_json = {
        let _graveyard = graveyard.lock().await;
        _graveyard.json()
    };

    println!(
        "{}",
        to_string_pretty(&graveyard_json).expect("serde_json::Value should serialize")
    );
}
