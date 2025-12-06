use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::communicative::tcp;
use crate::communicative::tcp::tcp::open_port;
use crate::communicative::tcp::tcp::port_number;
use crate::inscriptive::epoch::dir::EpochDirectory;
use crate::inscriptive::epoch::dir::EPOCH_DIRECTORY;
use crate::inscriptive::lp::dir::LPDirectory;
use crate::inscriptive::lp::dir::LP_DIRECTORY;
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
use crate::transmutative::noist::manager::DKGManager;
use crate::transmutative::noist::manager::DKG_MANAGER;
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
    let epoch_dir: EPOCH_DIRECTORY = match EpochDirectory::new(chain) {
        Some(epoch_dir) => epoch_dir,
        None => {
            println!("{}", "Error initializing epoch directory.".red());
            return;
        }
    };

    // #3 Initialize LP directory.
    let lp_dir: LP_DIRECTORY = match LPDirectory::new(chain) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing LP directory.".red());
            return;
        }
    };

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
        let epoch_dir = Arc::clone(&epoch_dir);
        let lp_dir = Arc::clone(&lp_dir);
        let registery = Arc::clone(&registery);
        let sync_manager = Arc::clone(&sync_manager);
        let coin_set = Arc::clone(&coin_set);

        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_sync_task(
                    chain,
                    &rpc_holder,
                    &key_holder,
                    &epoch_dir,
                    &lp_dir,
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
    let account = {
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
    {
        let is_lp = {
            let _lp_dir = lp_dir.lock().await;
            _lp_dir.is_lp(account)
        };

        let is_operator = {
            let _epoch_dir = epoch_dir.lock().await;
            _epoch_dir.is_operator(account)
        };

        if !is_lp && !is_operator {
            eprintln!(
                "{}",
                "This account is not an active liquidity provider or operator.".red()
            );
            return;
        }
    }

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
    let mut dkg_manager: DKG_MANAGER = match DKGManager::new(&lp_dir) {
        Some(manager) => manager,
        None => return eprintln!("{}", "Error initializing DKG manager.".red()),
    };

    // #17 Run TCP server.
    {
        let nns_client = nns_client.clone();
        let dkg_manager = Arc::clone(&dkg_manager);

        let _ = tokio::spawn(async move {
            let _ =
                tcp::server::run(mode, chain, &nns_client, &key_holder, &dkg_manager, None).await;
        });
    }

    // #18 CLI.
    cli(&mut dkg_manager, &coordinator).await;
}

pub async fn cli(dkg_manager: &mut DKG_MANAGER, coordinator: &PEER) {
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
            "dkg" => ocli::dkg::dkg_command(parts, coordinator, dkg_manager).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
