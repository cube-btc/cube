use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{ConfigResponseBody, TCPClient};
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::inscriptive::params_manager::params_manager::PARAMS_MANAGER;
use crate::inscriptive::registry::registry::REGISTRY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use serde_json::to_string_pretty;

/// config [sak <secondary_aggregation_key_hex>] [pc <projector_config_32b_hex>] [fc <flame_config_hex_bytes>]
pub async fn config_command(
    secondary_aggregation_key: Option<Vec<u8>>,
    projector_config: Option<[u8; 32]>,
    flame_config: Option<FMAccountFlameConfig>,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    registry: &REGISTRY,
    coin_manager: &COIN_MANAGER,
    params_manager: &PARAMS_MANAGER,
    engine_peer: &PEER,
) {
    if secondary_aggregation_key.is_none() && projector_config.is_none() && flame_config.is_none() {
        println!(
            "{}",
            "Error: provide at least one of sak, pc, fc.".red()
        );
        return;
    }

    let root_account = crate::constructive::entity::account::root_account::root_account::RootAccount::self_root_account_from_registry(key_holder, registry).await;
    let account_key = root_account.account_key();

    let batch_height_tip: u64 = {
        let _sync_manager = sync_manager.lock().await;
        _sync_manager.cube_batch_sync_height_tip()
    };
    let current_execution_batch_height = batch_height_tip + 1;
    let target = Target::new(current_execution_batch_height);

    let config = Config::new(
        root_account,
        secondary_aggregation_key,
        projector_config,
        flame_config,
        target,
    );

    let params_holder = {
        let _params_manager = params_manager.lock().unwrap();
        _params_manager.get_params_holder()
    };
    let config_bytes_len = config
        .secondary_aggregation_key
        .as_ref()
        .map(|v| v.len() as u64)
        .unwrap_or(0)
        + if config.projector_config.is_some() { 32 } else { 0 }
        + config
            .flame_config
            .as_ref()
            .map(|cfg| cfg.to_bytes().len() as u64)
            .unwrap_or(0);
    let pre_subsidy_fee = match params_holder.config_entry_base_fee.checked_add(
        config_bytes_len.saturating_mul(params_holder.config_entry_per_config_byte_fee),
    ) {
        Some(v) => v,
        None => {
            println!("{}", "Error: config fee overflow.".red());
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
    if account_balance < pre_subsidy_fee {
        println!(
            "{}",
            format!(
                "Insufficient local balance for config: balance={}, required={} (pre-subsidy).",
                account_balance, pre_subsidy_fee
            )
            .red()
        );
        return;
    }

    let config_bls_signature: [u8; 96] = match config.bls_sign(key_holder) {
        Ok(signature) => signature,
        Err(error) => {
            println!("{}", format!("Error BLS signing config: {:?}", error).red());
            return;
        }
    };

    let (config_response_body, duration) = match engine_peer
        .request_config(&config, config_bls_signature)
        .await
    {
        Ok((config_response_body, duration)) => (config_response_body, duration),
        Err(error) => {
            println!("{}", format!("Error requesting config: {:?}", error).red());
            return;
        }
    };

    match config_response_body {
        ConfigResponseBody::Ok(success_body) => {
            println!(
                "{}",
                format!(
                    "Config entry successfully executed ({} ms): {}",
                    duration.as_millis(),
                    to_string_pretty(&success_body.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        ConfigResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error executing config: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
