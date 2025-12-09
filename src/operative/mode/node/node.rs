use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::constructive::entity::account::account::Account;
use crate::inscriptive::registery_manager::registery_manager::RegisteryManager;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use crate::inscriptive::set::set::CoinSet;
use crate::inscriptive::set::set::COIN_SET;
use crate::inscriptive::sync_manager::sync_manager::SyncManager;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::wallet::wallet::Wallet;
use crate::inscriptive::wallet::wallet::WALLET;
use crate::operative::mode::ncli;
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
    let _operating_mode = OperatingMode::Node;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing node.");

    // #2 Initialize  wallet.
    let wallet: WALLET = match Wallet::new(chain, key_holder.public_key()) {
        Some(wallet) => wallet,
        None => {
            println!("{}", "Error initializing wallet.".red());
            return;
        }
    };

    // #3 Initialize Epoch directory.

    // #4 Initialize LP directory.

    // #5 Initialize Registery manager.
    let registery: REGISTERY_MANAGER = match RegisteryManager::new(chain) {
        Ok(registery_manager) => registery_manager,
        Err(_) => {
            println!("{}", "Error initializing registery manager.".red());
            return;
        }
    };

    // #6 Initialize the coin set.
    let coin_set: COIN_SET = match CoinSet::new(chain) {
        Some(coin_set) => coin_set,
        None => {
            println!("{}", "Error initializing coin set.".red());
            return;
        }
    };

    // #7 Initialize rollup directory.
    let sync_manager: SYNC_MANAGER = match SyncManager::new(chain) {
        Ok(sync_manager) => sync_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing sync manager: ".red(), err);
            return;
        }
    };

    // #8 Spawn syncer
    {
        let chain = chain.clone();
        let key_holder = key_holder.clone();
        let rpc_holder = rpc_holder.clone();

        let registery = Arc::clone(&registery);
        let wallet = Arc::clone(&wallet);
        let sync_manager = Arc::clone(&sync_manager);
        let coin_set = Arc::clone(&coin_set);

        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_sync_task(
                    chain,
                    &rpc_holder,
                    &key_holder,
                    &registery,
                    Some(&wallet),
                    &coin_set,
                )
                .await;
        });
    }

    println!("{}", "Syncing rollup.");

    // #9 Wait until rollup to be synced to the latest Bitcoin chain tip.
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

    // #11 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // #12 Connect to the coordinator.
    let coordinator: PEER = {
        let coordinator_key = coordinator_key(chain);

        loop {
            match Peer::connect(chain, PeerKind::Coordinator, coordinator_key, &nns_client).await {
                Ok(connection) => break connection,
                Err(_) => {
                    println!("{}", "Failed to connect. Re-trying in 5..".red());
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
        }
    };

    // #13 CLI.
    cli(chain, &coordinator, &key_holder, &account, &wallet).await;
}

pub async fn cli(
    _chain: Chain,
    coordinator_conn: &PEER,
    key_holder: &KeyHolder,
    _account: &Account,
    wallet: &WALLET,
) {
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
            "clear" => ncli::clear::clear_command(),
            "conn" => ncli::conn::conn_command(coordinator_conn).await,
            "ping" => ncli::ping::ping_command(coordinator_conn).await,
            "npub" => ncli::npub::npub_command(key_holder).await,
            "decomp" => ncli::decomp::decomp_command(parts),
            "move" => {
                ncli::r#move::move_command(
                    coordinator_conn,
                    wallet,
                    key_holder.secret_key(),
                    key_holder.public_key(),
                )
                .await
            }
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
