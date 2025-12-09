use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::manager::PEER_MANAGER;
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
use crate::operative::mode::ccli;
use crate::operative::sync::sync::RollupSync;
use crate::operative::Chain;
use crate::operative::OperatingKind;
use crate::operative::OperatingMode;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::io::{self, BufRead};
use std::sync::Arc;

#[tokio::main]
pub async fn run(
    key_holder: KeyHolder,
    chain: Chain,
    rpc_holder: BitcoinRPCHolder,
    _operating_mode: OperatingMode,
) {
    let operating_kind = OperatingKind::Coordinator;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing coordinator.");

    // #4 Initialize Registery.
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

    // #10 Check if this is the coordinator.
    if key_holder.public_key().serialize_xonly() != coordinator_key(chain) {
        eprintln!("{}", "Coordinator <nsec> does not match.".red());
        return;
    }

    // #11 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // #12 Open port 6272 for incoming connections.
    match open_port(chain).await {
        true => println!(
            "{}",
            format!("Opened port '{}'.", port_number(chain)).green()
        ),
        false => (),
    }

    // #13 Run NNS server.
    {
        let nns_client = nns_client.clone();
        let _ = tokio::spawn(async move {
            let _ = nns::server::run(&nns_client, operating_kind).await;
        });
    }

    // #14 Initialize peer manager.

    // #15 Initialize DKG Manager.

    // #16 Run background preprocessing for the DKG Manager.

    // #18 Construct CSession.

    // #20 Run TCP server.

    // #21 Initialize CLI.
}

pub async fn cli(peer_manager: &mut PEER_MANAGER) {
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
            "clear" => ccli::clear::clear_command(),
            "ops" => ccli::ops::ops_command(peer_manager).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
