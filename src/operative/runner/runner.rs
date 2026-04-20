use crate::communicative::nns;
use crate::communicative::nns::client::NNSClient;
use crate::communicative::peer::manager::engine_key;
use crate::communicative::peer::peer::Peer;
use crate::communicative::peer::peer::PeerKind;
use crate::communicative::peer::peer::PEER;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc::validate_rpc;
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::communicative::tcp::server as tcp_server;
use crate::communicative::tcp::tcp::open_port;
use crate::communicative::tcp::tcp::port_number;
use crate::inscriptive::archival_manager::archival_manager::ArchivalManager;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::CoinManager;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FlameManager;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::Graveyard;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::Registery;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::StateManager;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SyncManager;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXOSet;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::cli::cli::run_engine_cli;
use crate::operative::cli::cli::run_node_cli;
use crate::operative::run_args::{
    chain::Chain, operating_kind::OperatingKind, resource_mode::ResourceMode, sync_mode::SyncMode,
};
use crate::operative::tasks::chain_sync::chain_sync::ChainSync;
use crate::operative::tasks::engine_session::engine_session::engine_batch_builder_background_task;
use crate::operative::tasks::engine_session::session_pool::session_pool::SessionPool;
use crate::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
use crate::operative::tasks::in_flight_batch_sync::in_flight_batch_sync::in_flight_batch_sync_background_task;
use crate::transmutative::key::KeyHolder;
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;

/// Whether MuSig2-based interactive lifts are enabled. Set to false for now since it's not supported yet.
const V2_LIFT_ENABLED: bool = false;

#[tokio::main]
pub async fn run(
    resource_mode: ResourceMode,
    chain: Chain,
    operating_kind: OperatingKind,
    rpc_holder: BitcoinRPCHolder,
    sync_mode: SyncMode,
    key_holder: KeyHolder,
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
    let registery: REGISTERY = match Registery::new(chain) {
        Ok(registery) => registery,
        Err(_) => {
            println!("{}", "Error initializing registery.".red());
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

    // 6.b Initialize archival manager when running in archival resource mode.
    let archival_manager: Option<ARCHIVAL_MANAGER> = match resource_mode {
        ResourceMode::Archival => match ArchivalManager::new(chain) {
            Ok(m) => Some(m),
            Err(err) => {
                println!(
                    "{} {:?}",
                    "Error initializing archival manager: ".red(),
                    err
                );
                return;
            }
        },
        ResourceMode::Pruned => None,
    };

    // 7 Initialize utxo set.
    let utxo_set: UTXO_SET = match UTXOSet::new(chain) {
        Some(utxo_set) => utxo_set,
        None => {
            println!("{}", "Error initializing utxo set.".red());
            return;
        }
    };

    // 8 Initialize graveyard.
    let graveyard: GRAVEYARD = match Graveyard::new(chain) {
        Ok(graveyard) => graveyard,
        Err(err) => {
            println!("{} {:?}", "Error initializing graveyard: ".red(), err);
            return;
        }
    };

    // 9 Initialize coin manager.
    let coin_manager: COIN_MANAGER = match CoinManager::new(chain) {
        Ok(coin_manager) => coin_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing coin manager: ".red(), err);
            return;
        }
    };

    // 10 Initialize flame manager.
    let flame_manager: FLAME_MANAGER = match FlameManager::new(chain) {
        Ok(flame_manager) => flame_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing flame manager: ".red(), err);
            return;
        }
    };

    // 10.b Initialize state manager.
    let state_manager: STATE_MANAGER = match StateManager::new(chain) {
        Ok(state_manager) => state_manager,
        Err(err) => {
            println!("{} {:?}", "Error initializing state manager: ".red(), err);
            return;
        }
    };

    // 10.c Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // 10.d For node mode, pre-connect to engine so chain sync can pull batch containers.
    let pre_sync_engine_conn: Option<PEER> = match operating_kind {
        OperatingKind::Node => Some(loop {
            match Peer::connect(chain, PeerKind::Engine, engine_key, &nns_client).await {
                Ok(connection) => break connection,
                Err(_) => {
                    println!("{}", "Failed to connect. Re-trying in 5..".red());
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
        }),
        OperatingKind::Engine => None,
    };

    // 8 Spawn chain syncer to sync Bitcoin blocks.
    {
        let chain = chain.clone();
        let rpc_holder = rpc_holder.clone();
        let engine_conn = pre_sync_engine_conn.clone();
        let engine_key = engine_key;
        let registery = Arc::clone(&registery);
        let graveyard = Arc::clone(&graveyard);
        let coin_manager = Arc::clone(&coin_manager);
        let flame_manager = Arc::clone(&flame_manager);
        let state_manager = Arc::clone(&state_manager);
        let archival_manager = archival_manager.clone();
        let sync_manager = Arc::clone(&sync_manager);
        let utxo_set = Arc::clone(&utxo_set);
        tokio::spawn(async move {
            let _ = sync_manager
                .spawn_background_chain_syncer(
                    chain,
                    &rpc_holder,
                    &engine_conn,
                    engine_key,
                    &registery,
                    &graveyard,
                    &coin_manager,
                    &flame_manager,
                    &state_manager,
                    &archival_manager,
                    &utxo_set,
                )
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

            // 11.a.4 Construct session pool.
            let session_pool: SESSION_POOL = SessionPool::construct(
                engine_key,
                &sync_manager,
                &utxo_set,
                &registery,
                &graveyard,
                &coin_manager,
                &flame_manager,
                &state_manager,
                archival_manager.clone(),
            );

            // 11.a.5 Spawn engine batch builder background task.
            {
                let session_pool = Arc::clone(&session_pool);
                let sync_manager = Arc::clone(&sync_manager);
                let rpc_holder = rpc_holder.clone();
                let engine_key = engine_key.clone();
                let utxo_set = Arc::clone(&utxo_set);
                let registery = Arc::clone(&registery);
                let graveyard = Arc::clone(&graveyard);
                let coin_manager = Arc::clone(&coin_manager);
                let flame_manager = Arc::clone(&flame_manager);
                let state_manager = Arc::clone(&state_manager);
                let archival_manager = archival_manager.clone();
                let key_holder = Arc::clone(&key_holder);

                let _ = tokio::spawn(async move {
                    let _ = engine_batch_builder_background_task(
                        &session_pool,
                        &sync_manager,
                        &rpc_holder,
                        &key_holder,
                        engine_key,
                        &utxo_set,
                        &registery,
                        &graveyard,
                        &coin_manager,
                        &flame_manager,
                        &state_manager,
                        &archival_manager,
                    )
                    .await;
                });
            }

            // 11.a.6 Run the TCP server in the background.
            {
                let keys = Arc::clone(&key_holder);
                let chain = chain.clone();
                let session_pool = Arc::clone(&session_pool);
                let _ = tokio::spawn(async move {
                    tcp_server::server::run(operating_kind, chain, keys, &session_pool).await;
                });
            }

            // 11.a.7 Run the session in the background: TODO

            // 11.a.8 Run the Engine CLI.
            run_engine_cli(
                &session_pool,
                &sync_manager,
                &registery,
                &graveyard,
                &coin_manager,
                &flame_manager,
                &key_holder,
            )
            .await;
        }
        // 11.b Node-specific initializations.
        OperatingKind::Node => {
            // 11.b.1 Validate the node key.
            if self_account_key == engine_key {
                eprintln!("{}", "Engine cannot be run in node mode.".red());
                return;
            }

            // 11.b.2 Connect to the engine.
            let engine_conn: PEER =
                pre_sync_engine_conn.expect("Node mode must pre-connect to engine");

            // 11.b.3 Run the in-flight batch syncer in the background.
            if sync_mode == SyncMode::InFlight {
                let engine_conn = Arc::clone(&engine_conn);
                let sync_manager = Arc::clone(&sync_manager);
                let utxo_set = Arc::clone(&utxo_set);
                let registery = Arc::clone(&registery);
                let graveyard = Arc::clone(&graveyard);
                let coin_manager = Arc::clone(&coin_manager);
                let flame_manager = Arc::clone(&flame_manager);
                let state_manager = Arc::clone(&state_manager);
                let archival_manager = archival_manager.clone();

                tokio::spawn(async move {
                    in_flight_batch_sync_background_task(
                        &engine_conn,
                        &sync_manager,
                        engine_key,
                        &utxo_set,
                        &registery,
                        &graveyard,
                        &coin_manager,
                        &flame_manager,
                        &state_manager,
                        &archival_manager,
                    )
                    .await;
                });
            }

            // 11.b.4 Run the node CLI.
            run_node_cli(
                chain,
                engine_key,
                self_account_key,
                V2_LIFT_ENABLED,
                &engine_conn,
                &key_holder,
                &utxo_set,
                &sync_manager,
                &registery,
                &graveyard,
                &coin_manager,
                &flame_manager,
                &state_manager,
                archival_manager.clone(),
            )
            .await;
        }
    }
}
