use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::coordinator_key;
use crate::communicative::peer::manager::PeerManager;
use crate::communicative::peer::manager::PEER_MANAGER;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::communicative::tcp;
use crate::communicative::tcp::tcp::open_port;
use crate::communicative::tcp::tcp::port_number;
use crate::inscriptive::blacklist::BlacklistDirectory;
use crate::inscriptive::blacklist::BLIST_DIRECTORY;
use crate::inscriptive::epoch::dir::EpochDirectory;
use crate::inscriptive::epoch::dir::EPOCH_DIRECTORY;
use crate::inscriptive::lp::dir::LPDirectory;
use crate::inscriptive::lp::dir::LP_DIRECTORY;
use crate::inscriptive::registery::registery::Registery;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::rollup::dir::RollupDirectory;
use crate::inscriptive::rollup::dir::ROLLUP_DIRECTORY;
use crate::inscriptive::set::set::CoinSet;
use crate::inscriptive::set::set::COIN_SET;
use crate::operative::mode::ccli;
use crate::operative::mode::coordinator::dkgops::DKGOps;
use crate::operative::session::ccontext::CContextRunner;
use crate::operative::session::ccontext::CSessionCtx;
use crate::operative::session::ccontext::CSESSION_CTX;
use crate::operative::sync::sync::RollupSync;
use crate::operative::Chain;
use crate::operative::OperatingMode;
use crate::transmutative::key::KeyHolder;
use crate::transmutative::noist::manager::DKGManager;
use crate::transmutative::noist::manager::DKG_MANAGER;
use crate::transmutative::secp::into::IntoPointByteVec;
use colored::Colorize;
use std::io::{self, BufRead};
use std::sync::Arc;

#[tokio::main]
pub async fn run(key_holder: KeyHolder, chain: Chain, rpc_holder: BitcoinRPCHolder) {
    let mode = OperatingMode::Coordinator;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing coordinator.");

    // #2 Initialize Epoch directory.
    let epoch_dir: EPOCH_DIRECTORY = match EpochDirectory::new(chain) {
        Some(dir) => dir,
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

    // #4 Initialize Registery.
    let registery: REGISTERY = match Registery::new(chain) {
        Ok(dir) => dir,
        Err(_) => {
            println!("{}", "Error initializing registery.".red());
            return;
        }
    };

    // #6 Initialize rollup directory.
    let rollup_dir: ROLLUP_DIRECTORY = match RollupDirectory::new(chain) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing rollup directory.".red());
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
                    None,
                    &coin_set,
                )
                .await;
        });
    }

    println!("{}", "Syncing rollup.");

    // #9 Await rollup to be fully synced.
    rollup_dir.await_ibd().await;

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
            let _ = nns::server::run(&nns_client, mode).await;
        });
    }

    // #14 Initialize peer manager.
    let operator_set = {
        let _epoch_dir = epoch_dir.lock().await;
        _epoch_dir.operator_set().into_xpoint_vec().expect("")
    };
    let mut peer_manager: PEER_MANAGER =
        match PeerManager::new(chain, &nns_client, PeerKind::Operator, &operator_set).await {
            Some(manager) => manager,
            None => return eprintln!("{}", "Error initializing Peer manager.".red()),
        };

    // #15 Initialize DKG Manager.
    let mut dkg_manager: DKG_MANAGER = match DKGManager::new(&lp_dir) {
        Some(manager) => manager,
        None => return eprintln!("{}", "Error initializing DKG manager.".red()),
    };

    // #16 Run background preprocessing for the DKG Manager.
    dkg_manager.run_preprocessing(&mut peer_manager).await;

    // #17 Construct blacklist directory.
    let mut blacklist_dir: BLIST_DIRECTORY = match BlacklistDirectory::new(chain) {
        Some(blacklist_dir) => blacklist_dir,
        None => {
            eprintln!(
                "{}",
                "Unexpected error: Failed to create blaming directory.".red()
            );
            return;
        }
    };

    // #18 Construct CSession.
    let csession_ctx: CSESSION_CTX =
        CSessionCtx::construct(&dkg_manager, &peer_manager, &blacklist_dir, &registery);

    // #19 Run CSession.
    {
        let csession_ctx = Arc::clone(&csession_ctx);
        let _ = tokio::spawn(async move {
            csession_ctx.run().await;
        });
    }

    // #20 Run TCP server.
    {
        let nns_client = nns_client.clone();
        let dkg_manager = Arc::clone(&dkg_manager);
        let csession_ctx = Arc::clone(&csession_ctx);

        let _ = tokio::spawn(async move {
            let _ = tcp::server::run(
                mode,
                chain,
                &nns_client,
                &key_holder,
                &dkg_manager,
                Some(csession_ctx),
            )
            .await;
        });
    }

    // #21 Initialize CLI.
    cli(&mut peer_manager, &mut dkg_manager, &mut blacklist_dir).await;
}

pub async fn cli(
    peer_manager: &mut PEER_MANAGER,
    dkg_manager: &mut DKG_MANAGER,
    blacklist_dir: &mut BLIST_DIRECTORY,
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
            "clear" => ccli::clear::clear_command(),
            "dkg" => ccli::dkg::dkg_command(parts, peer_manager, dkg_manager).await,
            "ops" => ccli::ops::ops_command(peer_manager).await,
            "blist" => ccli::blist::blist_command(parts, blacklist_dir).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
