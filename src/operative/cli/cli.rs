use crate::communicative::peer::peer::PEER;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::cli::commands::common_commands;
use crate::operative::cli::commands::node_commands;
use crate::operative::run_args::chain::Chain;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::io;
use std::io::BufRead;

/// Runs the Engine CLI.
pub async fn run_engine_cli(_session_pool: &SESSION_POOL) {
    // 1 Print the CLI prompt.
    print_cli_prompt();

    // 2 Read the CLI input.
    let stdin = io::stdin();
    let handle = stdin.lock();

    // 3 Parse the CLI input.
    for line in handle.lines() {
        // 3.1 Parse the CLI input.
        let parts = match parse_cli_parts(line) {
            Some(parts) => parts,
            None => continue,
        };

        // 3.2 Match the CLI input.
        match parts[0].as_str() {
            // Main commands:
            "exit" => break,
            "clear" => common_commands::clear::clear_command(),
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}

/// Runs the Node CLI.
pub async fn run_node_cli(
    chain: Chain,
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    v2_lift_enabled: bool,
    _engine_conn: &PEER,
    key_holder: &KeyHolder,
    utxo_set: &UTXO_SET,
    sync_manager: &SYNC_MANAGER,
    registery: &REGISTERY,
    graveyard: &GRAVEYARD,
    coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
    state_manager: &STATE_MANAGER,
    archival_manager: Option<ARCHIVAL_MANAGER>,
) {
    // 1 Print the CLI prompt.
    print_cli_prompt();

    // 2 Read the CLI input.
    let stdin = io::stdin();
    let handle = stdin.lock();

    // 3 Parse the CLI input.
    for line in handle.lines() {
        // 3.1 Parse the CLI input.
        let parts = match parse_cli_parts(line) {
            Some(parts) => parts,
            None => continue,
        };

        // 3.2 Match the CLI input.
        match parts[0].as_str() {
            // Main commands:
            "exit" => break,
            "clear" => common_commands::clear::clear_command(),
            // Lift-Liftup related commands:
            "liftaddr" => {
                node_commands::liftaddr::liftaddr_command(chain, engine_key, self_account_key)
            }
            "lifts" => {
                node_commands::lifts::lifts_command(
                    engine_key,
                    self_account_key,
                    v2_lift_enabled,
                    utxo_set,
                )
                .await
            }
            "lift" => {
                node_commands::lift::lift_command(
                    engine_key,
                    self_account_key,
                    v2_lift_enabled,
                    key_holder,
                    sync_manager,
                    utxo_set,
                    registery,
                    graveyard,
                    coin_manager,
                    flame_manager,
                    state_manager,
                    archival_manager.clone(),
                )
                .await
            }
            //"conn" => ncli::conn::conn_command(engine_conn).await,
            //"ping" => ncli::ping::ping_command(engine_conn).await,
            "npub" => node_commands::npub::npub_command(key_holder).await,
            "decompile" => {
                let parts_ref: Vec<&str> = parts.iter().map(String::as_str).collect();
                node_commands::decompile::decompile_command(parts_ref);
            }
            //"move" => ncli::r#move::move_command(engine_conn, key_holder).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}

/// Prints the CLI prompt.
fn print_cli_prompt() {
    println!(
        "{}",
        "Enter command (type help for options, type exit to quit):".cyan()
    );
}

/// Parses the CLI input into parts.
fn parse_cli_parts(line: Result<String, io::Error>) -> Option<Vec<String>> {
    // 1 Parse the CLI input.
    let line = match line {
        Ok(line) => line,
        Err(_) => {
            eprintln!("{}", format!("Invalid line.").yellow());
            return None;
        }
    };

    // 2 Split the CLI input into parts.
    let parts: Vec<String> = line.split_whitespace().map(str::to_string).collect();
    if parts.is_empty() {
        return None;
    }

    Some(parts)
}
