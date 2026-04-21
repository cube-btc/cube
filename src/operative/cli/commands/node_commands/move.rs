use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{MoveResponseBody, TCPClient};
use crate::constructive::core_types::entities::account::account::account::Account;
use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use serde_json::to_string_pretty;

/// move <satoshi_amount> <to_account_key_hex>
pub async fn move_command(
    satoshi_amount: u32,
    to_account_key: [u8; 32],
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    registery: &REGISTERY,
    engine_peer: &PEER,
) {
    // 1 Construct sender root account.
    let from = RootAccount::self_root_account_from_registery(key_holder, registery).await;

    // 2 Construct receiver account from registery state.
    let to = Account::account_from_registery(to_account_key, registery).await;

    // 3 Get the current cube batch height tip from sync manager.
    let batch_height_tip: u64 = {
        let _sync_manager = sync_manager.lock().await;
        _sync_manager.cube_batch_sync_height_tip()
    };

    // 4 Current execution batch height is tip plus one.
    let current_execution_batch_height = batch_height_tip + 1;

    // 5 Construct target and move entry.
    let target = Target::new(current_execution_batch_height);
    let move_entry = Move::new(from, to, satoshi_amount, target);

    // 6 Sign move.
    let move_bls_signature: [u8; 96] = match move_entry.bls_sign(key_holder) {
        Ok(signature) => signature,
        Err(error) => {
            println!("{}", format!("Error BLS signing move: {:?}", error).red());
            return;
        }
    };

    // 7 Submit move request.
    let (move_response_body, duration) = match engine_peer
        .request_move(&move_entry, move_bls_signature)
        .await
    {
        Ok((move_response_body, duration)) => (move_response_body, duration),
        Err(error) => {
            println!("{}", format!("Error requesting move: {:?}", error).red());
            return;
        }
    };

    // 8 Print response.
    match move_response_body {
        MoveResponseBody::Ok(success_body) => {
            println!(
                "{}",
                format!(
                    "Move entry successfully executed ({} ms): {}",
                    duration.as_millis(),
                    to_string_pretty(&success_body.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        MoveResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error executing move: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
