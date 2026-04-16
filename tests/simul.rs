#[cfg(test)]
mod simul_tests {
    use bitcoin::hashes::Hash;
    use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};
    use cube::constructive::core_types::entities::account::root_account::root_account::RootAccount;
    use cube::constructive::core_types::target::target::Target;
    use cube::constructive::entries::entry_kinds::liftup::liftup::Liftup;
    use cube::constructive::txo::lift::lift::Lift;
    use cube::constructive::txo::lift::lift_versions::liftv1::liftv1::return_liftv1_scriptpubkey;
    use cube::executive::exec_ctx::exec_ctx::ExecCtx;
    use cube::executive::exec_ctx::exec_ctx::EXEC_CTX;
    use cube::inscriptive::coin_manager::coin_manager::erase_coin_manager;
    use cube::inscriptive::coin_manager::coin_manager::CoinManager;
    use cube::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
    use cube::inscriptive::flame_manager::flame_manager::erase_flame_manager;
    use cube::inscriptive::flame_manager::flame_manager::FlameManager;
    use cube::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
    use cube::inscriptive::graveyard::graveyard::erase_graveyard;
    use cube::inscriptive::graveyard::graveyard::Graveyard;
    use cube::inscriptive::graveyard::graveyard::GRAVEYARD;
    use cube::inscriptive::registery::registery::erase_registery;
    use cube::inscriptive::registery::registery::Registery;
    use cube::inscriptive::registery::registery::REGISTERY;
    use cube::inscriptive::sync_manager::sync_manager::erase_sync_manager;
    use cube::inscriptive::sync_manager::sync_manager::SyncManager;
    use cube::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
    use cube::inscriptive::utxo_set::utxo_set::erase_utxo_set;
    use cube::inscriptive::utxo_set::utxo_set::UTXOSet;
    use cube::inscriptive::utxo_set::utxo_set::UTXO_SET;
    use cube::operative::run_args::chain::Chain;
    use cube::operative::tasks::engine_session::session_pool::session_pool::SessionPool;
    use cube::operative::tasks::engine_session::session_pool::session_pool::SESSION_POOL;
    use cube::transmutative::key::KeyHolder;
    use hex;
    use serde_json::to_string_pretty;
    use std::sync::Arc;

    #[tokio::test]
    async fn liftup_simulation_single() -> Result<(), String> {
        // 1 Construct engine key.
        let engine_key: [u8; 32] = [
            0xa3, 0x08, 0xf8, 0x7d, 0x88, 0x7d, 0x78, 0x34, 0x19, 0xb8, 0x4b, 0x97, 0x65, 0x1f,
            0xd8, 0xa5, 0xf8, 0x8f, 0x6d, 0xb6, 0x41, 0x4a, 0xe6, 0xeb, 0x19, 0x84, 0xcc, 0x67,
            0x42, 0xee, 0xf0, 0x9e,
        ];

        // 2 Construct batch height, timestamp, and payload version.
        let (batch_height, batch_timestamp, payload_version) = (1, 1776015147, 1);

        // 3 Construct self secret key.
        let secret_key: [u8; 32] =
            hex::decode("2795044ce0f83f718bc79c5f2add1e52521978df91ce9b7f82c9097191d33602")
                .map_err(|_| format!("Failed to parse secret key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert secret key hex.".to_string())?;

        // 4 Construct self public key.
        let public_key: [u8; 32] =
            hex::decode("d0ea35e4a5d654109aef6b175672ea98099212a42d028fcf8bd4e38c137ff15a")
                .map_err(|_| format!("Failed to parse public key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert public key hex.".to_string())?;

        // 5 Construct self key holder.
        let key_holder = KeyHolder::new(secret_key).expect("Failed to create key holder.");

        // 6 Assert that the public key matches.
        assert_eq!(
            key_holder.secp_public_key_bytes(),
            public_key,
            "Public key mismatch."
        );

        // 7 Construct chain.
        let chain = Chain::Testbed;

        // 8 Erase and construct the sync manager.
        erase_sync_manager(chain);
        let _sync_manager: SYNC_MANAGER =
            SyncManager::new(chain).expect("Failed to create sync manager.");

        // 9 Erase and construct the UTXO set.
        erase_utxo_set(chain);
        let utxo_set: UTXO_SET = UTXOSet::new(chain).expect("Failed to create utxo set.");

        // 10 Erase and construct the registery.
        erase_registery(chain);
        let registery: REGISTERY = Registery::new(chain).expect("Failed to create registery.");

        // 11 Erase and construct the graveyard.
        erase_graveyard(chain);
        let graveyard: GRAVEYARD = Graveyard::new(chain).expect("Failed to create graveyard.");

        // 12 Erase and construct the coin manager.
        erase_coin_manager(chain);
        let coin_manager: COIN_MANAGER =
            CoinManager::new(chain).expect("Failed to create coin manager.");

        // 13 Erase and construct the flame manager.
        erase_flame_manager(chain);
        let flame_manager: FLAME_MANAGER =
            FlameManager::new(chain).expect("Failed to create flame manager.");

        // 14 Deposit some BTC: 10_000 satoshis.
        let lift: Lift = {
            // 14.1 Construct the Lift scriptpubkey/address to fund:
            let lift_scriptpubkey = return_liftv1_scriptpubkey(public_key, engine_key)
                .expect("Failed to construct the Lift scriptpubkey/address to fund.");

            // 14.2 Placeholder outpoint.
            let txid = Txid::from_byte_array([0x00u8; 32]);
            let vout = 0;
            let outpoint: OutPoint = OutPoint::new(txid, vout);

            // 14.3 Construct the TxOut.
            let txout = TxOut {
                value: Amount::from_sat(10_000),
                script_pubkey: ScriptBuf::from(lift_scriptpubkey),
            };

            // 14.4 Add the TxOut to the UTXO set.
            let mut _utxo_set = utxo_set.lock().await;
            _utxo_set.insert_utxo(&outpoint, &txout);

            // 14.5 Construct the Lift.
            Lift::new_liftv1(public_key, engine_key, outpoint, txout)
        };

        // 15 Construct Liftup.
        let liftup: Liftup = {
            // 15.1 Construct the Root Account.
            let root_account = RootAccount::self_root_account(&key_holder, &registery).await;

            // 15.2 Construct the Target aimed at the Engine's batch height.
            let target = Target::new(batch_height);

            // 15.3 Construct the Liftup.
            Liftup::new(root_account, target, vec![lift])
        };

        // 16 BLS sign the Liftup.
        let liftup_bls_signature = liftup
            .bls_sign(key_holder)
            .expect("Failed to BLS sign the Liftup.");

        // Prints
        {
            println!(
                "Liftup: {}",
                to_string_pretty(&liftup.json()).expect("serde_json::Value should serialize")
            );

            // Print the registery json nice.
            println!(
                "Registery: {}",
                to_string_pretty(&registery.lock().await.json())
                    .expect("serde_json::Value should serialize")
            );

            // Print the coin manager json nice.
            println!(
                "Coin Manager: {}",
                to_string_pretty(&coin_manager.lock().await.json())
                    .expect("serde_json::Value should serialize")
            );
        }

        // 17 Construct session pool.
        let session_pool: SESSION_POOL = SessionPool::construct(
            engine_key,
            &Arc::clone(&utxo_set),
            &Arc::clone(&registery),
            &Arc::clone(&graveyard),
            &Arc::clone(&coin_manager),
            &Arc::clone(&flame_manager),
        );

        // 18 Begin the session.
        {
            // 18.1 Lock the session pool.
            let mut _session_pool = session_pool.lock().await;

            // 18.2 Begin the session of the session pool.
            _session_pool.begin_session();
        }

        // 19 Execute the liftup in the session pool.
        let _liftup_entry = session_pool
            .lock()
            .await
            .exec_liftup_in_pool(batch_height, batch_timestamp, &liftup, liftup_bls_signature)
            .await
            .map_err(|error| {
                format!(
                    "Failed to execute the liftup in the session pool: {:?}",
                    error
                )
            })?;

        // 20 Convert the session pool to a batch template.
        let batch_template = session_pool
            .lock()
            .await
            .into_batch_template(batch_height, batch_timestamp, payload_version, 0)
            .await
            .map_err(|error| {
                format!(
                    "Failed to convert the session pool to a batch template: {:?}",
                    error
                )
            })?;

        // 21 Flush the session pool.
        {
            // 21.1 Lock the session pool.
            let mut _session_pool = session_pool.lock().await;

            // 21.2 Flush the session pool.
            _session_pool.flush().await;
        }

        // 22 Drop the session pool.
        drop(session_pool);

        // Now that we have the batch template, we consider this delivered to the Node from the Engine for execution.
        // So we better drop the session pool now and contstruct an ExecCtx to execute the batch.

        // 23 Construct an ExecCtx.
        let exec_ctx: EXEC_CTX = ExecCtx::construct(
            engine_key,
            Arc::clone(&utxo_set),
            Arc::clone(&registery),
            Arc::clone(&graveyard),
            Arc::clone(&coin_manager),
            Arc::clone(&flame_manager),
        );

        println!("pre batch execution prints:");

        // print the batch template
        println!(
            "Batch template: {}",
            to_string_pretty(&batch_template.json()).expect("serde_json::Value should serialize")
        );

        let prev_payload_tx_outpoint = OutPoint {
            txid: Txid::from_byte_array([0x00u8; 32]),
            vout: 0,
        };

        let new_payload_txout = TxOut {
            value: Amount::from_sat(0),
            script_pubkey: ScriptBuf::from(vec![0x00u8; 32]),
        };

        let mut bitcoin_tx_inputs = Vec::<OutPoint>::new();
        let mut bitcoin_tx_outputs = Vec::<TxOut>::new();

        bitcoin_tx_inputs.push(prev_payload_tx_outpoint);
        bitcoin_tx_outputs.push(new_payload_txout);

        bitcoin_tx_inputs.extend(batch_template.bitcoin_tx_inputs);
        bitcoin_tx_outputs.extend(batch_template.bitcoin_tx_outputs);

        // 24 Execute the batch.
        let (
            batch_height,
            payload_version,
            batch_timestamp,
            aggregate_bls_signature,
            executed_entries,
        ) = exec_ctx
            .lock()
            .await
            .execute_batch(
                batch_height,
                batch_template.payload_bytes,
                bitcoin_tx_inputs,
                bitcoin_tx_outputs,
            )
            .await
            .map_err(|error| format!("Failed to execute the batch: {:?}", error))?;

        println!("post batch execution prints:");

        // Post-execution Prints
        {
            println!("Post-execution Prints:");
            println!("Batch height: {}", batch_height);
            println!("Payload version: {}", payload_version);
            println!("Batch timestamp: {}", batch_timestamp);
            println!(
                "Aggregate BLS signature: {}",
                hex::encode(aggregate_bls_signature)
            );
            println!("Executed entries len: {}", executed_entries.len());
        }

        // 25 Apply changes.
        {
            // 25.1 Lock the exec ctx.
            let mut _exec_ctx = exec_ctx.lock().await;

            // 25.2 Apply the changes to the exec ctx.
            _exec_ctx
                .apply_changes(batch_height, batch_timestamp)
                .await
                .map_err(|error| {
                    format!("Failed to apply the changes to the exec ctx: {:?}", error)
                })?;
        }

        // Print managers.
        {
            println!("Post-execution Manager Prints:");
            println!(
                "Registery: {}",
                to_string_pretty(&registery.lock().await.json())
                    .expect("serde_json::Value should serialize")
            );
            println!(
                "Coin Manager: {}",
                to_string_pretty(&coin_manager.lock().await.json())
                    .expect("serde_json::Value should serialize")
            );
        }

        // 26 Return OK.
        Ok(())
    }
}
