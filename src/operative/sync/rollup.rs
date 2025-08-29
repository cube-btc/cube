use crate::{
    communicative::rpc::bitcoin_rpc::{
        bitcoin_rpc::{get_chain_tip, retrieve_block},
        bitcoin_rpc_holder::BitcoinRPCHolder,
    },
    constructive::{taproot::P2TR, txo::lift::Lift},
    inscriptive::{
        baked, epoch::dir::EPOCH_DIRECTORY, lp::dir::LP_DIRECTORY, registery::registery::REGISTERY,
        rollup::dir::ROLLUP_DIRECTORY, set::set::COIN_SET, wallet::wallet::WALLET,
    },
    operative::Chain,
    transmutative::key::KeyHolder,
};
use async_trait::async_trait;
use bitcoin::OutPoint;
use secp::Point;
use std::time::Duration;
use tokio::time::sleep;

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
    /// Continuously syncs the rollup.
    async fn sync(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        key_holder: &KeyHolder,
        _epoch_dir: &EPOCH_DIRECTORY,
        _lp_dir: &LP_DIRECTORY,
        _registery: &REGISTERY,
        wallet: Option<&WALLET>,
        coin_set: &COIN_SET,
    );

    /// Awaits the rollup to be synced to the latest Bitcoin chain tip.
    async fn await_sync(&self);
}

#[async_trait]
impl RollupSync for ROLLUP_DIRECTORY {
    async fn await_sync(&self) {
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

    async fn sync(
        &self,
        chain: Chain,
        rpc_holder: &BitcoinRPCHolder,
        key_holder: &KeyHolder,
        epoch_dir: &EPOCH_DIRECTORY,
        _lp_dir: &LP_DIRECTORY,
        _registery: &REGISTERY,
        wallet: Option<&WALLET>,
        coin_set: &COIN_SET,
    ) {
        let mut synced: bool = false;

        let rollup_dir: &ROLLUP_DIRECTORY = self;

        let sync_start_height = match chain {
            Chain::Signet => baked::SIGNET_SYNC_START_HEIGHT,
            Chain::Mainnet => baked::MAINNET_SYNC_START_HEIGHT,
        };

        loop {
            let rollup_bitcoin_sync_height = {
                let _rollup_dir = rollup_dir.lock().await;
                _rollup_dir.bitcoin_sync_height()
            };

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

            // Retrieve chain height.
            let chain_height = {
                match get_chain_tip(rpc_holder) {
                    Ok(tip) => tip,
                    Err(_) => {
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            };

            match rollup_bitcoin_sync_height == chain_height {
                true => {
                    // Rollup is fully synced.
                    if !synced {
                        {
                            let mut _rollup_dir = rollup_dir.lock().await;
                            _rollup_dir.set_synced(true);
                        }
                        synced = true;
                    }

                    sleep(Duration::from_secs(10)).await;
                }
                false => {
                    // Rollup is not fully synced.
                    let height_to_sync = match rollup_bitcoin_sync_height < sync_start_height {
                        true => sync_start_height,
                        false => rollup_bitcoin_sync_height + 1,
                    };

                    let block = match retrieve_block(rpc_holder, height_to_sync) {
                        Ok(block) => block,
                        Err(_) => {
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    };

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
                        let mut _rollup_dir = rollup_dir.lock().await;
                        _rollup_dir.set_bitcoin_sync_height(height_to_sync);
                    }

                    // TODO set the new rollup sync height.

                    println!("Synced height #{}.", height_to_sync);
                }
            }
        }
    }
}
