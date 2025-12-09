use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::communicative::tcp::tcp::open_port;
use crate::communicative::tcp::tcp::port_number;
use crate::inscriptive::registery_manager::registery_manager::RegisteryManager;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use crate::inscriptive::set::set::CoinSet;
use crate::inscriptive::set::set::COIN_SET;
use crate::inscriptive::sync_manager::sync_manager::SyncManager;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::operative::mode::ocli;
use crate::operative::sync::sync::RollupSync;
use crate::operative::Chain;
use crate::operative::OperatingMode;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
pub async fn run(key_holder: KeyHolder, chain: Chain, rpc_holder: BitcoinRPCHolder) {
    let mode = OperatingMode::Operator;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing operator..");

    // #2 Initialize Epoch directory.

    // #3 Initialize LP directory.

    // #4 Initialize Registery manager.
    let registery: REGISTERY_MANAGER = match RegisteryManager::new(chain) {
        Ok(registery_manager) => registery_manager,
        Err(_) => {
            println!("{}", "Error initializing registery manager.".red());
            return;
        }
    };

    // #6 Initialize rollup directory.
    let sync_manager: SYNC_MANAGER = match SyncManager::new(chain) {
        Ok(sync_manager) => sync_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing sync manager: ".red(), err);
            return;
        }
    };

    // #7 Initialize the coin set.
    let coin_set: COIN_SET = match CoinSet::new(chain) {
        Some(coin_set) => coin_set,
        None => {
            println!("{}", "Error initializing coin set.".red());
            return;
        }
    };

    // #8 Spawn syncer.
    {
        let chain = chain.clone();
        let key_holder = key_holder.clone();
        let rpc_holder = rpc_holder.clone();
        let registery = Arc::clone(&registery);
        let sync_manager = Arc::clone(&sync_manager);
        let coin_set = Arc::clone(&coin_set);

        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_sync_task(
                    chain,
                    &rpc_holder,
                    &key_holder,
                    &registery,
                    None,
                    &coin_set,
                )
                .await;
        });
    }

    println!("{}", "Syncing rollup.");

    // #9 Await rollup to be fully synced.
    sync_manager.await_ibd().await;

    println!("{}", "Syncing complete.");

    // #10 Construct account.
    let _account = {
        let _registery_manager = registery.lock().await;

        match _registery_manager.get_account_by_key(key_holder.public_key().serialize_xonly()) {
            Some(account) => account,
            None => {
                println!("{}", "Error constructing account.".red());
                return;
            }
        }
    };

    // #11 Check if this account is a liquidity provider or an operator.

    // #12 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // #13 Open port 6272 for incoming connections.
    match open_port(chain).await {
        true => println!(
            "{}",
            format!("Opened port '{}'.", port_number(chain)).green()
        ),
        false => (),
    }

    // #14 Run NNS server.
    {
        let nns_client = nns_client.clone();
        let _ = tokio::spawn(async move {
            let _ = nns::server::run(&nns_client, mode).await;
        });
    }

    // #15 Connect to the coordinator.
    let coordinator: PEER = {
        let coordinator_key = coordinator_key(chain);

        loop {
            match Peer::connect(chain, PeerKind::Coordinator, coordinator_key, &nns_client).await {
                Ok(connection) => break connection,
                Err(_) => {
                    println!(
                        "{}",
                        "Failed to connect coordinator. Re-trying in 5..".red()
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
        }
    };

    // #16 Initialize DKG Manager.

    // #17 Run TCP server.

    // #18 CLI.
    cli(&coordinator).await;
}

pub async fn cli(_coordinator: &PEER) {
    println!(
        "{}",
        "Enter command (type help for options, type exit to quit):".cyan()
    );

    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => {
                eprintln!("{}", format!("Invalid line.").yellow());
                continue;
            }
        };

        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            // Main commands:
            "exit" => break,
            "clear" => ocli::clear::clear_command(),
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
