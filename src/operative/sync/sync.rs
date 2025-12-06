use crate::{
    communicative::rpc::bitcoin_rpc::{
        bitcoin_rpc::{get_chain_tip, retrieve_block},
        bitcoin_rpc_holder::BitcoinRPCHolder,
    },
    constructive::{taproot::P2TR, txo::lift::Lift},
    inscriptive::{
        baked, epoch::dir::EPOCH_DIRECTORY, lp::dir::LP_DIRECTORY,
        registery_manager::registery_manager::REGISTERY_MANAGER, set::set::COIN_SET,
        sync_manager::sync_manager::SYNC_MANAGER, wallet::wallet::WALLET,
    },
    operative::Chain,
    transmutative::key::KeyHolder,
};
use async_trait::async_trait;
use bitcoin::OutPoint;
use colored::Colorize;
use secp::Point;
use std::time::Duration;
use tokio::time::sleep;

/// Number of blocks a block needs to be buried to be considered final.
/// This will require 6 on-chain confirmations for a transaction to be considered final.
const BLOCK_DEPTH_FOR_FINALITY: u64 = 5;

type LiftSPK = Vec<u8>;

/// Returns the list of Taproot scriptpubkeys to scan.
pub async fn lifts_spks_to_scan(
    key_holder: &KeyHolder,
    epoch_dir: &EPOCH_DIRECTORY,
) -> Option<Vec<(LiftSPK, Point)>> {
    let mut spks = Vec::<(LiftSPK, Point)>::new();

    let self_key = key_holder.public_key();

    let group_keys = {
        let _epoch_dir = epoch_dir.lock().await;
        _epoch_dir.active_group_keys()
    };

    for operator_group_key in group_keys.iter() {
        let lift = Lift::new(self_key, operator_group_key.to_owned(), None, None);
        let taproot = lift.taproot()?;
        let spk = taproot.spk()?;

        spks.push((spk, operator_group_key.to_owned()));
    }

    Some(spks)
}

#[async_trait]
pub trait RollupSync {
    /// Spawns a background task to continuously sync the rollup.
    async fn spawn_background_sync_task(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        key_holder: &KeyHolder,
        _epoch_dir: &EPOCH_DIRECTORY,
        _lp_dir: &LP_DIRECTORY,
        _registery: &REGISTERY_MANAGER,
        wallet: Option<&WALLET>,
        coin_set: &COIN_SET,
    );

    /// Awaits the rollup to be fully synced to the latest chain tip.
    async fn await_ibd(&self);
}

#[async_trait]
impl RollupSync for SYNC_MANAGER {
    async fn await_ibd(&self) {
        loop {
            let is_fully_synced = {
                let _self = self.lock().await;
                _self.is_synced()
            };

            match is_fully_synced {
                true => break,
                false => sleep(Duration::from_secs(5)).await,
            }
        }
    }

    async fn spawn_background_sync_task(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        key_holder: &KeyHolder,
        epoch_dir: &EPOCH_DIRECTORY,
        _lp_dir: &LP_DIRECTORY,
        _registery: &REGISTERY_MANAGER,
        wallet: Option<&WALLET>,
        coin_set: &COIN_SET,
    ) {
        let mut synced: bool = false;

        let sync_manager: &SYNC_MANAGER = self;

        let sync_start_height = match chain {
            Chain::Signet | Chain::Testbed => baked::SIGNET_SYNC_START_HEIGHT,
            Chain::Mainnet => baked::MAINNET_SYNC_START_HEIGHT,
        };

        // Initialize the Bitcoin node's chain tip.
        let mut bitcoin_node_chain_tip;

        // Retrieve Bitcoin node's chain tip.
        loop {
            match get_chain_tip(rpc_holder) {
                Ok((tip, is_synced)) => {
                    // Check if the Bitcoin node is fully synced.
                    match is_synced {
                        true => {
                            bitcoin_node_chain_tip = tip;
                            break;
                        }
                        false => {
                            // Sleep and retry.
                            sleep(Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                }
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!(
                            "Error retrieving Bitcoin node's chain tip: {}. Retrying in 5s...",
                            err
                        )
                        .yellow()
                    );

                    // Sleep and retry.
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        }

        // Print the Bitcoin node's chain tip.
        println!("Bitcoin chain tip: #{}", bitcoin_node_chain_tip);

        'outer_sync_iteration: loop {
            // Retrieve cube node's sync height.
            let cube_node_sync_height = {
                let _sync_manager = sync_manager.lock().await;
                _sync_manager.bitcoin_sync_height()
            };

            // Retrieve self lifts.
            let self_lifts = {
                match wallet {
                    Some(wallet) => {
                        let lift_wallet = {
                            let _wallet = wallet.lock().await;
                            _wallet.lift_wallet()
                        };

                        let _lift_wallet = lift_wallet.lock().await;
                        _lift_wallet.lifts()
                    }
                    None => vec![],
                }
            };

            // The target sync height is the latest Bitcoin chain tip minus BLOCK_DEPTH_FOR_FINALITY.
            let target_sync_height = bitcoin_node_chain_tip - BLOCK_DEPTH_FOR_FINALITY;

            // Check if cube node is fully synced.
            match cube_node_sync_height == target_sync_height {
                true => {
                    // Check for a new block.
                    'check_for_a_new_block: loop {
                        match get_chain_tip(rpc_holder) {
                            Ok((new_tip, _)) => {
                                // Check if the chain tip has changed.
                                match new_tip > bitcoin_node_chain_tip {
                                    // A new block was mined.
                                    true => {
                                        // Update the chain tip.
                                        bitcoin_node_chain_tip = new_tip;

                                        // Print the new chain tip.
                                        println!("New Bitcoin chain tip: #{}", new_tip);

                                        // Stop checking for a new block.
                                        break 'check_for_a_new_block;
                                    }
                                    // No new block was mined (or possibly a small reorg if the new tip is smaller).
                                    false => {
                                        // Check if the cube node is fully synced.
                                        if !synced {
                                            {
                                                // Set the rollup to synced.
                                                let mut _sync_manager = sync_manager.lock().await;
                                                _sync_manager.set_synced(true);
                                            }

                                            // Set the synced flag.
                                            synced = true;

                                            // Print the status update.
                                            println!("{}", "Node is fully synced.".green());
                                        }

                                        // Sleep for 10s.
                                        sleep(Duration::from_secs(10)).await;

                                        // Continue checking for a new block.
                                        continue 'check_for_a_new_block;
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "{}",
                                    format!(
                                        "Error retrieving chain tip: {}. Retrying in 5s...",
                                        err
                                    )
                                    .yellow()
                                );

                                // Sleep and retry.
                                sleep(Duration::from_secs(5)).await;
                                continue 'check_for_a_new_block;
                            }
                        };
                    }

                    // Continue the loop.
                    continue 'outer_sync_iteration;
                }
                false => {
                    // Cube node is not fully synced.
                    let height_to_sync = match cube_node_sync_height < sync_start_height {
                        true => sync_start_height,
                        false => cube_node_sync_height + 1,
                    };

                    // Retrieve the block.
                    let block = match retrieve_block(rpc_holder, height_to_sync) {
                        Ok(block) => block,
                        Err(err) => {
                            // Print the error.
                            eprintln!(
                                "{}",
                                format!(
                                    "Retrieve block error at height #{}: {}. Retrying in 5s...",
                                    height_to_sync, err
                                )
                                .yellow()
                            );

                            // Sleep and retry.
                            sleep(Duration::from_secs(5)).await;
                            continue 'outer_sync_iteration;
                        }
                    };

                    // Retrieve the lift spks to scan.
                    let lift_spks_to_scan = match wallet {
                        Some(_) => match lifts_spks_to_scan(key_holder, epoch_dir).await {
                            Some(spks) => spks,
                            None => vec![],
                        },
                        None => vec![],
                    };

                    // Scan block..
                    for transaction in block.txdata.iter() {
                        let inputs = transaction.input.clone();
                        let outputs = transaction.output.clone();
                        let txid = transaction.compute_txid();

                        // Iterate over inputs.
                        for txn_input in inputs.iter() {
                            let txn_input_outpoint = txn_input.previous_output;

                            // Remove spent lifts from wallet.
                            if let Some(wallet) = wallet {
                                // Compare to self lift outpoints.
                                for lift in self_lifts.iter() {
                                    if let Some(self_lift_outpoint) = lift.outpoint() {
                                        if txn_input_outpoint == self_lift_outpoint {
                                            // Remove from lift wallet.
                                            {
                                                let lift_wallet = {
                                                    let _wallet = wallet.lock().await;
                                                    _wallet.lift_wallet()
                                                };

                                                let mut _lift_wallet = lift_wallet.lock().await;
                                                _lift_wallet.remove_lift(lift);
                                            }
                                        }
                                    }
                                }
                            }

                            // Remove spent utxos from utxoset.
                            {
                                let utxo_set = {
                                    let _coin_set = coin_set.lock().await;
                                    _coin_set.utxo_set()
                                };

                                let mut _utxo_set = utxo_set.lock().await;
                                _utxo_set.remove_txout(&txn_input_outpoint);
                            }
                        }

                        // Iterate over outputs.
                        for (txn_output_index, txn_output) in outputs.iter().enumerate() {
                            let txn_output_spk = txn_output.script_pubkey.as_bytes().to_vec();
                            let txn_output_val = txn_output.value.to_sat();
                            let txn_output_outpoint = OutPoint::new(txid, txn_output_index as u32);

                            // Compare to lift spks to scan.
                            if let Some(wallet) = wallet {
                                for (lift_spk, operator_group_key) in lift_spks_to_scan.iter() {
                                    if &txn_output_spk == lift_spk {
                                        let self_key = key_holder.public_key();
                                        let operator_key = operator_group_key.to_owned();

                                        let lift = Lift::new(
                                            self_key,
                                            operator_key,
                                            Some(txn_output_outpoint),
                                            Some(txn_output_val),
                                        );

                                        // Add to lift wallet.
                                        {
                                            let lift_wallet = {
                                                let _wallet = wallet.lock().await;
                                                _wallet.lift_wallet()
                                            };

                                            let mut _lift_wallet = lift_wallet.lock().await;
                                            _lift_wallet.insert_lift(&lift);
                                        }
                                    }
                                }
                            }

                            // Add to utxoset.
                            {
                                let utxo_set = {
                                    let _coin_set = coin_set.lock().await;
                                    _coin_set.utxo_set()
                                };

                                let mut _utxo_set = utxo_set.lock().await;
                                _utxo_set.insert_txout(&txn_output_outpoint, txn_output);
                            }
                        }
                    }

                    // Set the new rollup bitcoin sync height.
                    {
                        let mut _sync_manager = sync_manager.lock().await;
                        _sync_manager.set_bitcoin_sync_height(height_to_sync);
                    }

                    // TODO set the new rollup sync height.

                    println!("Synced height #{}.", height_to_sync);

                    // Continue the loop.
                    continue 'outer_sync_iteration;
                }
            }
        }
    }
}
