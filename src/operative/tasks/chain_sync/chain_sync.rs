use crate::{
    communicative::rpc::bitcoin_rpc::{
        bitcoin_rpc::{get_chain_tip, retrieve_block},
        bitcoin_rpc_holder::BitcoinRPCHolder,
    },
    inscriptive::{
        baked, registery::registery::REGISTERY, sync_manager::sync_manager::SYNC_MANAGER,
        utxo_set::utxo_set::UTXO_SET,
    },
    operative::run_args::chain::Chain,
};
use async_trait::async_trait;
use bitcoin::OutPoint;
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

/// Number of blocks a block needs to be buried to be considered final.
/// This will require 2 on-chain confirmations for a transaction to be considered final.
const BLOCK_DEPTH_FOR_FINALITY: u64 = 2;

#[async_trait]
pub trait ChainSync {
    /// Spawns a background task to continuously sync the chain.
    async fn spawn_background_chain_syncer(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        _registery: &REGISTERY,
        utxo_set: &UTXO_SET,
    );

    /// Awaits the chain to be fully synced to the latest chain tip.
    async fn await_ibd(&self);
}

#[async_trait]
impl ChainSync for SYNC_MANAGER {
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

    async fn spawn_background_chain_syncer(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        _registery: &REGISTERY,
        utxo_set: &UTXO_SET,
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
                _sync_manager.bitcoin_sync_height_tip()
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
                                                // Set the chain to synced.
                                                let mut _sync_manager = sync_manager.lock().await;
                                                _sync_manager.set_synced(true);
                                            }

                                            // Set the synced flag.
                                            synced = true;
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

                    // Scan block..
                    for transaction in block.txdata.iter() {
                        let inputs = transaction.input.clone();
                        let outputs = transaction.output.clone();
                        let txid = transaction.compute_txid();

                        // Iterate over inputs.
                        for txn_input in inputs.iter() {
                            let txn_input_outpoint = txn_input.previous_output;

                            // Remove spent utxos from utxoset.
                            {
                                let mut _utxo_set = utxo_set.lock().await;
                                _utxo_set.remove_utxo(&txn_input_outpoint);
                            }
                        }

                        // Iterate over outputs.
                        for (txn_output_index, txn_output) in outputs.iter().enumerate() {
                            let txn_output_outpoint = OutPoint::new(txid, txn_output_index as u32);

                            // Add to utxoset.
                            {
                                let mut _utxo_set = utxo_set.lock().await;
                                _utxo_set.insert_utxo(&txn_output_outpoint, txn_output);
                            }
                        }
                    }

                    // Set the new bitcoin sync height.
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
