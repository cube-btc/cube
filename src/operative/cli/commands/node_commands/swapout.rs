use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{SwapoutResponseBody, TCPClient};
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::params_manager::params_manager::PARAMS_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use serde_json::to_string_pretty;

/// swapout <amount>
pub async fn swapout_command(
    amount: u32,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    registery: &REGISTERY,
    coin_manager: &COIN_MANAGER,
    params_manager: &PARAMS_MANAGER,
    engine_peer: &PEER,
) {
    let root_account = RootAccount::self_root_account_from_registery(key_holder, registery).await;
    let account_key = root_account.account_key();

    let swapout_base_fee = {
        let _params_manager = params_manager.lock().unwrap();
        _params_manager.get_params_holder().swapout_entry_base_fee
    };

    let total_required = match u64::from(amount).checked_add(swapout_base_fee) {
        Some(value) => value,
        None => {
            println!("{}", "Error: amount + base fee overflows u64.".red());
            return;
        }
    };

    let account_balance = {
        let _coin_manager = coin_manager.lock().await;
        _coin_manager.get_account_balance(account_key)
    };

    let account_balance = match account_balance {
        Some(balance) => balance,
        None => {
            println!(
                "{}",
                "Error: no local coin-manager balance for self account.".red()
            );
            return;
        }
    };

    if account_balance < total_required {
        println!(
            "{}",
            format!(
                "Insufficient local balance for swapout: balance={}, required={} (amount {} + base_fee {}).",
                account_balance, total_required, amount, swapout_base_fee
            )
            .red()
        );
        return;
    }

    let batch_height_tip: u64 = {
        let _sync_manager = sync_manager.lock().await;
        _sync_manager.cube_batch_sync_height_tip()
    };
    let current_execution_batch_height = batch_height_tip + 1;
    let target = Target::new(current_execution_batch_height);

    let pinless_self = PinlessSelf::new_default(account_key, None);
    let swapout = Swapout::new(root_account, amount, target, pinless_self);

    let swapout_bls_signature: [u8; 96] = match swapout.bls_sign(key_holder) {
        Ok(signature) => signature,
        Err(error) => {
            println!("{}", format!("Error BLS signing swapout: {:?}", error).red());
            return;
        }
    };

    let (swapout_response_body, duration) = match engine_peer
        .request_swapout(&swapout, swapout_bls_signature)
        .await
    {
        Ok((swapout_response_body, duration)) => (swapout_response_body, duration),
        Err(error) => {
            println!("{}", format!("Error requesting swapout: {:?}", error).red());
            return;
        }
    };

    match swapout_response_body {
        SwapoutResponseBody::Ok(success_body) => {
            println!(
                "{}",
                format!(
                    "Swapout entry successfully executed ({} ms): {}",
                    duration.as_millis(),
                    to_string_pretty(&success_body.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        SwapoutResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error executing swapout: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
