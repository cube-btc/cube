use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::engine_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::communicative::tcp::tcp::open_port;
use crate::communicative::tcp::tcp::port_number;
use crate::inscriptive::registery_manager::registery_manager::RegisteryManager;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SyncManager;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXOSet;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::cli::cli::run_engine_cli;
use crate::operative::cli::cli::run_node_cli;
use crate::operative::tasks::chain_sync::chain_sync::ChainSync;
use crate::operative::Chain;
use crate::operative::OperatingKind;
use crate::operative::OperatingMode;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;

/// Whether MuSig2-based interactive lifts are enabled. Set to false for now since it's not supported yet.
const V2_LIFT_ENABLED: bool = false;

#[tokio::main]
pub async fn run(
    key_holder: KeyHolder,
    chain: Chain,
    rpc_holder: BitcoinRPCHolder,
    operating_kind: OperatingKind,
    _operating_mode: OperatingMode,
) {
    // 1 Wrap KeyHolder
    let key_holder = Arc::new(key_holder);

    // 2 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, chain) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    // 3 Print the initializing message according to the operating kind.
    match operating_kind {
        OperatingKind::Engine => {
            println!("{}", "Initializing engine.");
        }
        OperatingKind::Node => {
            println!("{}", "Initializing node.");
        }
    }

    // 4 Get the engine key and self account key.
    let (engine_key, self_account_key) = (engine_key(chain), key_holder.secp_public_key_bytes());

    // 5 Initialize registery.
    let registery: REGISTERY_MANAGER = match RegisteryManager::new(chain) {
        Ok(registery_manager) => registery_manager,
        Err(_) => {
            println!("{}", "Error initializing registery manager.".red());
            return;
        }
    };

    // 6 Initialize sync manager.
    let sync_manager: SYNC_MANAGER = match SyncManager::new(chain) {
        Ok(sync_manager) => sync_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing sync manager: ".red(), err);
            return;
        }
    };

    // 7 Initialize utxo set.
    let utxo_set: UTXO_SET = match UTXOSet::new(chain) {
        Some(utxo_set) => utxo_set,
        None => {
            println!("{}", "Error initializing utxo set.".red());
            return;
        }
    };

    // 8 Spawn chain syncer to sync Bitcoin blocks.
    {
        let chain = chain.clone();
        let rpc_holder = rpc_holder.clone();
        let registery = Arc::clone(&registery);
        let sync_manager = Arc::clone(&sync_manager);
        let utxo_set = Arc::clone(&utxo_set);
        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_chain_syncer(chain, &rpc_holder, &registery, &utxo_set)
                .await;
        });
    }

    // 9 Initial Block Download (IBD) encapsulation.
    {
        println!("{}", "Syncing chain.");

        // #9 Await chain to be fully synced.
        sync_manager.await_ibd().await;

        println!("{}", "Syncing complete.");
    }

    // 10 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // 11 Operating-kind-specific initializations.
    match operating_kind {
        // 11.a Engine-specific initializations.
        OperatingKind::Engine => {
            // 11.a.1 Validate the engine key.
            if self_account_key != engine_key {
                eprintln!("{}", "Engine <nsec> does not match with the Engine.".red());
                return;
            }

            // 11.a.2 Open port 6272 for incoming connections.
            match open_port(chain).await {
                true => println!(
                    "{}",
                    format!("Opened port '{}'.", port_number(chain)).green()
                ),
                false => (),
            }

            // 11.a.3 Run NNS server in the background.
            {
                let nns_client = nns_client.clone();
                let _ = tokio::spawn(async move {
                    let _ = nns::server::run(&nns_client, operating_kind).await;
                });
            }

            // 11.a.4 Run the TCP server in the background: TODO

            // 11.a.5 Run the session in the background: TODO

            // 11.a.6 Run the Engine CLI.
            run_engine_cli().await;
        }
        // 11.b Node-specific initializations.
        OperatingKind::Node => {
            // 11.b.1 Validate the node key.
            if self_account_key == engine_key {
                eprintln!("{}", "Engine cannot be run in node mode.".red());
                return;
            }

            // 11.b.2 Connect to the engine.
            let engine_conn: PEER = {
                loop {
                    match Peer::connect(chain, PeerKind::Engine, engine_key, &nns_client).await {
                        Ok(connection) => break connection,
                        Err(_) => {
                            println!("{}", "Failed to connect. Re-trying in 5..".red());
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    };
                }
            };

            // 11.b.3 Run the node CLI.
            run_node_cli(
                chain,
                engine_key,
                self_account_key,
                V2_LIFT_ENABLED,
                &engine_conn,
                &key_holder,
                &registery,
                &utxo_set,
            )
            .await;
        }
    }
}
