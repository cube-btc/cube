use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::{DeployResponseBody, TCPClient};
use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::executive::executable::compiler::compiler::ProgramCompiler;
use crate::executive::executable::executable::Program;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::params_manager::params_manager::PARAMS_MANAGER;
use crate::inscriptive::registry::registry::REGISTRY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use serde_json::to_string_pretty;

/// deploy <initial_balance> <0x program bytes>
pub async fn deploy_command(
    initial_balance: u32,
    program_bytes: Vec<u8>,
    key_holder: &KeyHolder,
    sync_manager: &SYNC_MANAGER,
    registry: &REGISTRY,
    coin_manager: &COIN_MANAGER,
    params_manager: &PARAMS_MANAGER,
    engine_peer: &PEER,
) {
    let root_account = RootAccount::self_root_account_from_registry(key_holder, registry).await;
    let account_key = root_account.account_key();

    let program = {
        let mut stream = program_bytes.clone().into_iter();
        match Program::decompile(&mut stream) {
            Ok(program) => program,
            Err(error) => {
                println!(
                    "{}",
                    format!("Error decompiling deploy program bytes: {error}").red()
                );
                return;
            }
        }
    };

    let batch_height_tip: u64 = {
        let _sync_manager = sync_manager.lock().await;
        _sync_manager.cube_batch_sync_height_tip()
    };
    let current_execution_batch_height = batch_height_tip + 1;
    let target = Target::new(current_execution_batch_height);

    let deploy = Deploy::new(root_account, program, initial_balance, target);

    let params_holder = {
        let _params_manager = params_manager.lock().unwrap();
        _params_manager.get_params_holder()
    };

    let pre_subsidy_fee = match params_holder.deploy_entry_base_fee.checked_add(
        (program_bytes.len() as u64).saturating_mul(params_holder.deploy_entry_per_program_byte_fee),
    ) {
        Some(v) => v,
        None => {
            println!("{}", "Error: deploy fee overflow.".red());
            return;
        }
    };

    let required_total = match pre_subsidy_fee.checked_add(initial_balance as u64) {
        Some(v) => v,
        None => {
            println!("{}", "Error: required deploy debit overflow.".red());
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

    if account_balance < required_total {
        println!(
            "{}",
            format!(
                "Insufficient local balance for deploy: balance={}, required={} (pre-subsidy + initial balance).",
                account_balance, required_total
            )
            .red()
        );
        return;
    }

    let deploy_bls_signature: [u8; 96] = match deploy.bls_sign(key_holder) {
        Ok(signature) => signature,
        Err(error) => {
            println!("{}", format!("Error BLS signing deploy: {:?}", error).red());
            return;
        }
    };

    let (deploy_response_body, duration) = match engine_peer
        .request_deploy(&deploy, deploy_bls_signature)
        .await
    {
        Ok((deploy_response_body, duration)) => (deploy_response_body, duration),
        Err(error) => {
            println!("{}", format!("Error requesting deploy: {:?}", error).red());
            return;
        }
    };

    match deploy_response_body {
        DeployResponseBody::Ok(success_body) => {
            println!(
                "{}",
                format!(
                    "Deploy entry successfully executed ({} ms): {}",
                    duration.as_millis(),
                    to_string_pretty(&success_body.json())
                        .expect("serde_json::Value should serialize")
                )
                .green()
            );
        }
        DeployResponseBody::Err(error) => {
            println!(
                "{}",
                format!(
                    "Error executing deploy: {}",
                    to_string_pretty(&error.json()).expect("serde_json::Value should serialize")
                )
                .red()
            );
        }
    }
}
