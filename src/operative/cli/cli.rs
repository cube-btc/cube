use crate::communicative::peer::peer::PEER;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::params_manager::params_manager::PARAMS_MANAGER;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
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
pub async fn run_engine_cli(
    _session_pool: &SESSION_POOL,
    chain: Chain,
    sync_manager: &SYNC_MANAGER,
    registery: &REGISTERY,
    graveyard: &GRAVEYARD,
    coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
    key_holder: &KeyHolder,
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
            "tip" => common_commands::tip::tip_command(sync_manager).await,
            "runexplorer" => {
                let port: u16 = match parts.get(1).and_then(|s| s.parse().ok()) {
                    Some(p) => p,
                    None => {
                        eprintln!("{}", "Usage: runexplorer <port> (e.g. 8080).".yellow());
                        continue;
                    }
                };
                common_commands::runexplorer::runexplorer_command(
                    chain,
                    port,
                    archival_manager.as_ref(),
                    registery,
                    coin_manager,
                    flame_manager,
                )
                .await;
            }
            "rootaccount" => common_commands::rootaccount::rootaccount_command(key_holder, registery).await,
            "engine" => common_commands::engine::engine_command(chain),
            "print" => match parts.get(1).map(String::as_str) {
                Some("registery") => common_commands::registery::registery_command(registery).await,
                Some("coinmanager") => {
                    common_commands::coinmanager::coinmanager_command(coin_manager).await
                }
                Some("graveyard") => common_commands::graveyard::graveyard_command(graveyard).await,
                Some("flamemanager") => {
                    common_commands::flamemanager::flamemanager_command(flame_manager).await
                }
                _ => eprintln!(
                    "{}",
                    "Usage: print <registery|coinmanager|graveyard|flamemanager>.".yellow()
                ),
            },
            "registery" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _registery = registery.lock().await;
                            _registery.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: registery isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
            "coinmanager" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _coin_manager = coin_manager.lock().await;
                            _coin_manager.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: coinmanager isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
            "flamemanager" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _flame_manager = flame_manager.lock().await;
                            _flame_manager.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: flamemanager isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
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
    engine_conn: &PEER,
    key_holder: &KeyHolder,
    utxo_set: &UTXO_SET,
    sync_manager: &SYNC_MANAGER,
    registery: &REGISTERY,
    graveyard: &GRAVEYARD,
    coin_manager: &COIN_MANAGER,
    flame_manager: &FLAME_MANAGER,
    state_manager: &STATE_MANAGER,
    privileges_manager: &PRIVILEGES_MANAGER,
    params_manager: &PARAMS_MANAGER,
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
            "tip" => common_commands::tip::tip_command(sync_manager).await,
            "runexplorer" => {
                let port: u16 = match parts.get(1).and_then(|s| s.parse().ok()) {
                    Some(p) => p,
                    None => {
                        eprintln!("{}", "Usage: runexplorer <port> (e.g. 8080).".yellow());
                        continue;
                    }
                };
                common_commands::runexplorer::runexplorer_command(
                    chain,
                    port,
                    archival_manager.as_ref(),
                    registery,
                    coin_manager,
                    flame_manager,
                )
                .await;
            }
            "rootaccount" => {
                common_commands::rootaccount::rootaccount_command(key_holder, registery).await
            }
            "engine" => common_commands::engine::engine_command(chain),
            "print" => match parts.get(1).map(String::as_str) {
                Some("registery") => common_commands::registery::registery_command(registery).await,
                Some("coinmanager") => {
                    common_commands::coinmanager::coinmanager_command(coin_manager).await
                }
                Some("graveyard") => common_commands::graveyard::graveyard_command(graveyard).await,
                Some("flamemanager") => {
                    common_commands::flamemanager::flamemanager_command(flame_manager).await
                }
                _ => eprintln!(
                    "{}",
                    "Usage: print <registery|coinmanager|graveyard|flamemanager>.".yellow()
                ),
            },
            "registery" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _registery = registery.lock().await;
                            _registery.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: registery isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
            "coinmanager" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _coin_manager = coin_manager.lock().await;
                            _coin_manager.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: coinmanager isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
            "flamemanager" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("isaccountregistered"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                                continue;
                            }
                        };

                        let is_registered = {
                            let _flame_manager = flame_manager.lock().await;
                            _flame_manager.is_account_registered(account_key)
                        };
                        println!("{}", is_registered);
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: flamemanager isaccountregistered <account_key_hex>.".yellow()
                        );
                    }
                }
            }
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
            "liftup" => {
                node_commands::liftup::liftup_command(
                    engine_key,
                    self_account_key,
                    v2_lift_enabled,
                    key_holder,
                    sync_manager,
                    utxo_set,
                    registery,
                    engine_conn,
                )
                .await
            }
            "batchrecord" => {
                let batch_height: u64 = match parts.get(1).and_then(|s| s.parse().ok()) {
                    Some(h) => h,
                    None => {
                        eprintln!(
                            "{}",
                            "Usage: batchrecord <batch_height> (non-negative integer)."
                                .yellow()
                        );
                        continue;
                    }
                };
                node_commands::batchrecord::batchrecord_command(batch_height, engine_conn).await;
            }
            "liftuplocal" => {
                node_commands::liftuplocal::liftup_local_command(
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
                    privileges_manager,
                    params_manager,
                    archival_manager.clone(),
                )
                .await
            }
            "conn" => node_commands::conn::conn_command(engine_conn).await,
            "ping" => node_commands::ping::ping_command(engine_conn).await,
            "npub" => node_commands::npub::npub_command(key_holder).await,
            "coins" => {
                let account_key = match parts.get(1).map(String::as_str) {
                    None => self_account_key,
                    Some(account_key_str) => match parse_account_key(account_key_str) {
                        Some(key) => key,
                        None => {
                            eprintln!("{}", "Invalid account key: expected 32-byte hex.".yellow());
                            continue;
                        }
                    },
                };
                node_commands::coins::coins_command(coin_manager, account_key).await;
            }
            "decompile" => {
                let parts_ref: Vec<&str> = parts.iter().map(String::as_str).collect();
                node_commands::decompile::decompile_command(parts_ref);
            }
            "account" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("rank"), Some(account_key_str)) => {
                        let account_key = match parse_account_key(account_key_str) {
                            Some(key) => key,
                            None => {
                                eprintln!(
                                    "{}",
                                    "Invalid account key: expected 32-byte hex.".yellow()
                                );
                                continue;
                            }
                        };
                        node_commands::rank::account_rank_command(registery, account_key).await;
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: account rank <account_key_hex>.".yellow()
                        );
                    }
                }
            }
            "contract" => {
                match (
                    parts.get(1).map(String::as_str),
                    parts.get(2).map(String::as_str),
                ) {
                    (Some("rank"), Some(contract_id_str)) => {
                        let contract_id = match parse_contract_id(contract_id_str) {
                            Some(id) => id,
                            None => {
                                eprintln!(
                                    "{}",
                                    "Invalid contract id: expected 32-byte hex.".yellow()
                                );
                                continue;
                            }
                        };
                        node_commands::rank::contract_rank_command(registery, contract_id).await;
                    }
                    _ => {
                        eprintln!(
                            "{}",
                            "Usage: contract rank <contract_id_hex>.".yellow()
                        );
                    }
                }
            }
            "move" => {
                let satoshi_amount: u32 = match parts.get(1).and_then(|s| s.parse().ok()) {
                    Some(amount) => amount,
                    None => {
                        eprintln!(
                            "{}",
                            "Usage: move <satoshi_amount> <to_account_key_hex>.".yellow()
                        );
                        continue;
                    }
                };

                let to_account_key: [u8; 32] = match parts.get(2) {
                    Some(account_key_str) => match parse_account_key(account_key_str) {
                        Some(account_key) => account_key,
                        None => {
                            eprintln!(
                                "{}",
                                "Invalid to account key: expected 32-byte hex.".yellow()
                            );
                            continue;
                        }
                    },
                    None => {
                        eprintln!(
                            "{}",
                            "Usage: move <satoshi_amount> <to_account_key_hex>.".yellow()
                        );
                        continue;
                    }
                };

                node_commands::r#move::move_command(
                    satoshi_amount,
                    to_account_key,
                    key_holder,
                    sync_manager,
                    registery,
                    engine_conn,
                )
                .await;
            }
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

fn parse_account_key(account_key_str: &str) -> Option<[u8; 32]> {
    parse_32_byte_hex(account_key_str)
}

fn parse_contract_id(contract_id_str: &str) -> Option<[u8; 32]> {
    parse_32_byte_hex(contract_id_str)
}

fn parse_32_byte_hex(s: &str) -> Option<[u8; 32]> {
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s).ok()?;
    bytes.try_into().ok()
}
