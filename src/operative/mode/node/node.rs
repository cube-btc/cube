use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::constructive::entity::account::account::Account;
use crate::inscriptive::epoch::dir::EpochDirectory;
use crate::inscriptive::epoch::dir::EPOCH_DIRECTORY;
use crate::inscriptive::lp::dir::LPDirectory;
use crate::inscriptive::lp::dir::LP_DIRECTORY;
use crate::inscriptive::registery::account_registery::account_registery::ACCOUNT_REGISTERY;
use crate::inscriptive::registery::registery::Registery;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::rollup::dir::RollupDirectory;
use crate::inscriptive::rollup::dir::ROLLUP_DIRECTORY;
use crate::inscriptive::set::set::CoinSet;
use crate::inscriptive::set::set::COIN_SET;
use crate::inscriptive::wallet::wallet::Wallet;
use crate::inscriptive::wallet::wallet::WALLET;
use crate::operative::mode::ncli;
use crate::operative::sync::rollup::RollupSync;
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
    let epoch_dir: EPOCH_DIRECTORY = match EpochDirectory::new(chain) {
        Some(epoch_dir) => epoch_dir,
        None => {
            println!("{}", "Error initializing epoch directory.".red());
            return;
        }
    };

    // #4 Initialize LP directory.
    let lp_dir: LP_DIRECTORY = match LPDirectory::new(chain) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing LP directory.".red());
            return;
        }
    };

    // #5 Initialize Registery.
    let registery: REGISTERY = match Registery::new(chain) {
        Ok(dir) => dir,
        Err(_) => {
            println!("{}", "Error initializing registery.".red());
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
    let rollup_dir: ROLLUP_DIRECTORY = match RollupDirectory::new(chain) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing rollup directory.".red());
            return;
        }
    };

    // #8 Spawn syncer
    {
        let chain = chain.clone();
        let key_holder = key_holder.clone();
        let rpc_holder = rpc_holder.clone();
        let epoch_dir = Arc::clone(&epoch_dir);
        let lp_dir = Arc::clone(&lp_dir);
        let registery = Arc::clone(&registery);
        let wallet = Arc::clone(&wallet);
        let rollup_dir = Arc::clone(&rollup_dir);
        let coin_set = Arc::clone(&coin_set);

        tokio::spawn(async move {
            let _ = rollup_dir
                .spawn_background_sync_task(
                    chain,
                    &rpc_holder,
                    &key_holder,
                    &epoch_dir,
                    &lp_dir,
                    &registery,
                    Some(&wallet),
                    &coin_set,
                )
                .await;
        });
    }

    println!("{}", "Syncing rollup.");

    // #9 Wait until rollup to be synced to the latest Bitcoin chain tip.
    rollup_dir.await_ibd().await;

    println!("{}", "Syncing complete.");

    // #10 Construct account.
    let account = {
        let account_registery: ACCOUNT_REGISTERY = {
            let _registery = registery.lock().await;
            _registery.account_registery()
        };

        let _account_registery = account_registery.lock().await;

        match _account_registery
            .get_account_by_account_key(key_holder.public_key().serialize_xonly())
        {
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
    cli(
        chain,
        &coordinator,
        &key_holder,
        &account,
        &wallet,
        &epoch_dir,
    )
    .await;
}

pub async fn cli(
    chain: Chain,
    coordinator_conn: &PEER,
    key_holder: &KeyHolder,
    _account: &Account,
    wallet: &WALLET,
    epoch_dir: &EPOCH_DIRECTORY,
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
            "addr" => ncli::addr::addr_command(chain, epoch_dir, key_holder).await,
            "lift" => ncli::lift::lift_command(wallet, epoch_dir, chain, key_holder, parts).await,
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
