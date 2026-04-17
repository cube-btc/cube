#[cfg(test)]
mod batchtxn_test {
    use bitcoin::hashes::Hash;
    use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};
    use cube::constructive::bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn;
    use cube::constructive::core_types::target::target::Target;
    use cube::constructive::entity::account::root_account::root_account::RootAccount;
    use cube::constructive::entry::entry::entry::Entry;
    use cube::constructive::entry::entry_kinds::liftup::liftup::Liftup;
    use cube::constructive::taproot::P2TR;
    use cube::constructive::txo::lift::lift::Lift;
    use cube::constructive::txo::lift::lift_versions::liftv1::liftv1::{
        return_liftv1_address, return_liftv1_scriptpubkey,
    };
    use cube::constructive::txout_types::payload::payload::Payload;
    use cube::inscriptive::registery::registery::erase_registery;
    use cube::inscriptive::registery::registery::Registery;
    use cube::inscriptive::registery::registery::REGISTERY;
    use cube::operative::run_args::chain::Chain;
    use cube::transmutative::codec::address::encode_p2tr;
    use cube::transmutative::key::KeyHolder;

    #[tokio::test]
    async fn batchtxn_test() -> Result<(), String> {
        // 1 Construct Engine secret key.
        let engine_secret_key: [u8; 32] =
            hex::decode("2b9906a26e64b48f8f94cf17e9681cf189c74b73d5fe69c2906550a2dcc33b5f")
                .map_err(|_| format!("Failed to parse secret key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert secret key hex.".to_string())?;

        // 2 Construct Engine public key.
        let engine_public_key: [u8; 32] =
            hex::decode("f437f28e3d9dc4638fe24699feeb89c094544163706faea65b2b72f91cb7267a")
                .map_err(|_| format!("Failed to parse public key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert public key hex.".to_string())?;

        // 3 Construct Engine key holder.
        let engine_key_holder =
            KeyHolder::new(engine_secret_key).expect("Failed to create key holder.");

        // 4 Assert that the public key matches.
        assert_eq!(
            engine_key_holder.secp_public_key_bytes(),
            engine_public_key,
            "Public key mismatch."
        );

        // 5 Construct User secret key.
        let user_secret_key: [u8; 32] =
            hex::decode("5280340afb7ade681b5d761b621818ef73ea6a10a425304d68d27a5d823df403")
                .map_err(|_| format!("Failed to parse secret key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert secret key hex.".to_string())?;

        // 6 Construct User public key.
        let user_public_key: [u8; 32] =
            hex::decode("06971ffe504c95152517d1be306b89b69e2f33c2c0e2a06b55f09d087b639d50")
                .map_err(|_| format!("Failed to parse public key hex."))?
                .try_into()
                .map_err(|_| "Failed to convert public key hex.".to_string())?;

        // 7 Construct User key holder.
        let user_key_holder =
            KeyHolder::new(user_secret_key).expect("Failed to create key holder.");

        // 8 Assert that the public key matches.
        assert_eq!(
            user_key_holder.secp_public_key_bytes(),
            user_public_key,
            "Public key mismatch."
        );

        // 9 Construct chain.
        let chain = Chain::Signet;

        // 10 Erase and construct the registery.
        erase_registery(chain);
        let registery: REGISTERY = Registery::new(chain).expect("Failed to create registery.");

        // 9 Construct payload bytes.
        let payload_bytes = vec![0xde, 0xad, 0xbe, 0xef];

        // 10 Construct payload to fund.
        let payload_to_fund = Payload::new(engine_public_key, payload_bytes.clone(), None);

        // 11 Get the taproot for the payload to fund.
        let payload_to_fund_taproot = payload_to_fund.taproot().expect("Failed to get taproot.");

        let payload_to_fund_scriptpubkey = payload_to_fund
            .calculated_scriptpubkey()
            .expect("Failed to get scriptpubkey.");

        println!(
            "Payload to fund scriptpubkey: {}",
            hex::encode(payload_to_fund_scriptpubkey.clone())
        );

        // 12 Get the tweaked key for the payload to fund.
        let payload_to_fund_taproot_key: [u8; 32] = payload_to_fund_taproot
            .tweaked_key()
            .expect("Failed to get tweaked key.")
            .serialize_xonly();

        // 13 Encode the tweaked key for the payload to fund into an address.
        let payload_to_fund_taproot_address =
            encode_p2tr(Chain::Signet, payload_to_fund_taproot_key)
                .expect("Failed to encode taproot address.");

        println!(
            "Payload to fund taproot address: {}",
            payload_to_fund_taproot_address
        );

        // Upon confirmation...

        // d1e3db2e7141123a9132976cda6285e6fdd10fe36bbd73fd724c8f4a989985de
        let prev_payload_txid: [u8; 32] =
            hex::decode("de8599984a8f4c72fd73bd6be30fd1fde68562da6c9732913a1241712edbe3d1")
                .map_err(|_| format!("Failed to parse prev payload txid hex."))?
                .try_into()
                .map_err(|_| "Failed to convert prev payload txid hex.".to_string())?;

        // Synthetic confirmed UTXO: must use this payload's P2TR scriptPubKey so spends match.
        let prev_payload_location = (
            OutPoint {
                txid: Txid::from_raw_hash(Hash::from_byte_array(prev_payload_txid)),
                vout: 0,
            },
            TxOut {
                value: Amount::from_sat(2000),
                script_pubkey: ScriptBuf::from(payload_to_fund_scriptpubkey),
            },
        );

        let prev_payload = Payload::new(
            engine_public_key,
            payload_bytes,
            Some(prev_payload_location),
        );

        let liftv1_address =
            return_liftv1_address(Chain::Signet, user_public_key, engine_public_key)
                .expect("Failed to get liftv1 address.");

        println!("Liftv1 address: {}", liftv1_address);

        // 28d024000e05c1683f11b3ea60623a2529ce20e75b019a3d3280236a5a1e625e
        let funded_lift_txid: [u8; 32] =
            hex::decode("5e621e5a6a2380323d9a015be720ce29253a6260eab3113f68c1050e0024d028")
                .map_err(|_| format!("Failed to parse prev payload txid hex."))?
                .try_into()
                .map_err(|_| "Failed to convert prev payload txid hex.".to_string())?;

        let liftv1_scriptpubkey = return_liftv1_scriptpubkey(user_public_key, engine_public_key)
            .expect("Failed to get liftv1 scriptpubkey.");

        println!(
            "Liftv1 scriptpubkey: {}",
            hex::encode(liftv1_scriptpubkey.clone())
        );

        let funded_lift_outpoint = OutPoint::new(
            Txid::from_raw_hash(Hash::from_byte_array(funded_lift_txid)),
            0,
        );
        let funded_lift_txout = TxOut {
            value: Amount::from_sat(1000),
            script_pubkey: ScriptBuf::from(liftv1_scriptpubkey.clone()),
        };

        let lift = Lift::new_liftv1(
            user_public_key,
            engine_public_key,
            funded_lift_outpoint,
            funded_lift_txout,
        );

        let liftup: Liftup = {
            // 15.1 Construct the Root Account.
            let root_account = RootAccount::self_root_account(&user_key_holder, &registery).await;

            // 15.2 Construct the Target aimed at the Engine's batch height.
            let target = Target::new(0);

            // 15.3 Construct the Liftup.
            Liftup::new(root_account, target, vec![lift])
        };

        let liftup_entry = Entry::Liftup(liftup);

        let new_payload_bytes = vec![0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe];

        let new_payload = Payload::new(engine_public_key, new_payload_bytes, None);

        let signed_batch_txn = SignedBatchTxn::construct(
            prev_payload,
            vec![],
            vec![liftup_entry],
            new_payload,
            None,
            500,
            &engine_key_holder,
        )
        .expect("Failed to construct signed batch transaction.");

        let signed_batch_txn_bytes = signed_batch_txn.serialize_bytes();

        println!(
            "Signed batch transaction bytes: {}",
            hex::encode(signed_batch_txn_bytes)
        );

        Ok(())
    }
}
