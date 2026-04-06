use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::engine_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::inscriptive::registery_manager::registery_manager::RegisteryManager;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SyncManager;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXOSet;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::mode::ncli;
use crate::operative::sync::sync::RollupSync;
use crate::operative::Chain;
use crate::operative::OperatingKind;
use crate::operative::OperatingMode;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::time::Duration;

/// Whether MuSig2-based interactive lifts are enabled. Set to false for now since it's not supported yet.
const V2_LIFT_ENABLED: bool = false;

#[tokio::main]
pub async fn run(
    key_holder: KeyHolder,
    chain: Chain,
    rpc_holder: BitcoinRPCHolder,
    _operating_mode: OperatingMode,
) {
    // Wrap KeyHolder in Arc for safe sharing across async tasks.
    // This avoids cloning secrets and maintains a single copy in memory.
    let key_holder = Arc::new(key_holder);
    let _operating_kind = OperatingKind::Node;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing node.");

    let engine_key = engine_key(chain);
    let self_account_key = key_holder.secp_public_key_bytes();

    // #3 Initialize Epoch directory.

    // #4 Initialize LP directory.

    // #5 Initialize Registery manager.
    let registery_manager: REGISTERY_MANAGER = match RegisteryManager::new(chain) {
        Ok(registery_manager) => registery_manager,
        Err(err) => {
            println!(
                "{} {:?}",
                "Error initializing registery manager: ".red(),
                err
            );
            return;
        }
    };

    // #6 Initialize the utxo set.
    let utxo_set: UTXO_SET = match UTXOSet::new(chain) {
        Some(utxo_set) => utxo_set,
        None => {
            println!("{}", "Error initializing utxo set: ".red());
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
        let rpc_holder = rpc_holder.clone();
        let registery_manager = Arc::clone(&registery_manager);
        let sync_manager = Arc::clone(&sync_manager);
        let utxo_set = Arc::clone(&utxo_set);

        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_sync_task(chain, &rpc_holder, &registery_manager, &utxo_set)
                .await;
        });
    }

    println!("{}", "Syncing Bitcoin.");

    // #9 Wait until Bitcoin to be synced to the latest Bitcoin chain tip.
    sync_manager.await_ibd().await;

    println!("{}", "Syncing complete.");

    // #11 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // #12 Connect to the coordinator.
    // let engine: PEER = {
    //    loop {
    //        match Peer::connect(chain, PeerKind::Engine, engine_key, &nns_client).await {
    //            Ok(connection) => break connection,
    //            Err(_) => {
    //                println!("{}", "Failed to connect. Re-trying in 5..".red());
    //                tokio::time::sleep(Duration::from_secs(5)).await;
    //                continue;
    //            }
    //        };
    //    }
    //  };

    // #13 CLI.
    cli(
        chain,
        engine_key,
        self_account_key,
        &key_holder,
        &registery_manager,
        &utxo_set,
    )
    .await;
}

pub async fn cli(
    chain: Chain,
    engine_key: [u8; 32],
    self_account_key: [u8; 32],
    //engine_conn: &PEER,
    key_holder: &KeyHolder,
    registery_manager: &REGISTERY_MANAGER,
    utxo_set: &UTXO_SET,
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
            // Lift-Liftup related commands:
            "liftaddr" => ncli::liftaddr::liftaddr_command(chain, engine_key, self_account_key),
            "lifts" => {
                ncli::lifts::lifts_command(engine_key, self_account_key, V2_LIFT_ENABLED, utxo_set)
                    .await
            }
            "lift" => {
                ncli::lift::lift_command(
                    engine_key,
                    self_account_key,
                    V2_LIFT_ENABLED,
                    key_holder,
                    utxo_set,
                    registery_manager,
                )
                .await
            }
            //"conn" => ncli::conn::conn_command(engine_conn).await,
            //"ping" => ncli::ping::ping_command(engine_conn).await,
            "npub" => ncli::npub::npub_command(key_holder).await,
            "decompile" => ncli::decompile::decompile_command(parts),
            //"move" => ncli::r#move::move_command(engine_conn, key_holder).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
