#[cfg(test)]
mod batchtxn_test {
    use bitcoin::hashes::Hash;
    use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};
    use cube::constructive::taproot::P2TR;
    use cube::constructive::txout_types::payload::payload::Payload;
    use cube::operative::run_args::chain::Chain;
    use cube::transmutative::codec::address::encode_p2tr;
    use cube::transmutative::key::KeyHolder;

    #[test]
    fn batchtxn_test() -> Result<(), String> {
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

        let prev_payload_txid: [u8; 32] =
            hex::decode("6934dce5becc4db24754bab80d4d9b667c88476e1438ed49b2ecab0fb5b16d47")
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

        Ok(())
    }
}
