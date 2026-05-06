use crate::constructive::txn::ext::OutpointExt;
use crate::transmutative::hash::{sha256, Hash, HashTag};
use crate::{
    constructive::bitcoiny::batch_txn::unsigned_batch_txn::error::construct_error::UnsignedBatchTxnConstructError,
    transmutative::codec::varint::encode_varint,
};
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};

// Bare transaction fields:
const N_VERSION: [u8; 4] = [0x02, 0x00, 0x00, 0x00];
const N_LOCKTIME: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

// BIP-341 SIGHASH_DEFAULT:
const TAPROOT_SIGHASH_DEFAULT: u8 = 0x00;

// BIP-342 hardcoded constants:
const TAPSCRIPT_DEFAULT_PUBLIC_KEY_VERSION: u8 = 0x00;
const TAPSCRIPT_DEFAULT_CODESEPARATOR_POS: u32 = 0xffff_ffff;

pub type TapLeafHash = [u8; 32];

/// Represents an unsigned batch transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedBatchTxn {
    /// The Bitcoin transaction inputs of the batch.
    pub tx_inputs: Vec<(OutPoint, TxOut)>,

    /// The Bitcoin transaction outputs of the batch.
    pub tx_outputs: Vec<TxOut>,
}

/// Represents an unsigned batch transaction constructor.
impl UnsignedBatchTxn {
    /// Constructs an unsigned batch transaction.
    pub fn construct(
        // Payload prevout
        prev_payload_tx_input: (OutPoint, TxOut),
        // Extra inputs for expired projectors
        extra_in_tx_inputs: Vec<(OutPoint, TxOut)>,
        // Entry lift tx inputs
        lift_tx_inputs: Vec<(OutPoint, TxOut)>,
        // Payload output
        new_payload_txout: TxOut,
        // Projector output
        new_projector_txout: Option<TxOut>,
        // Swapout outputs
        swapout_tx_outputs: Vec<TxOut>,
        // Bitcoin transaction feerate (sats per vbyte)
        bitcoin_transaction_feerate: u64,
    ) -> Result<UnsignedBatchTxn, UnsignedBatchTxnConstructError> {
        println!(
            "bitcoin_transaction_feerate: {}",
            bitcoin_transaction_feerate
        );

        // Keep rough input-kind counts for transaction vsize estimation.
        let lift_tx_inputs_len = lift_tx_inputs.len();

        // Initialzie the tx inputs.
        let mut tx_inputs = Vec::new();

        // Initialzie the tx outputs.
        let mut tx_outputs = Vec::new();

        // Initialize the tx inputs value sum.
        let mut tx_inputs_value_sum = 0;

        // Fill transaction inputs.
        {
            // Push the prev payload outpoint to the tx inputs.
            tx_inputs.push((prev_payload_tx_input.0, prev_payload_tx_input.1.clone()));

            // Add the prev payload value to the tx inputs value sum.
            tx_inputs_value_sum += prev_payload_tx_input.1.value.to_sat();

            // Push the extra inputs to the tx inputs.
            for (outpoint, txout) in extra_in_tx_inputs {
                tx_inputs.push((outpoint, txout.clone()));

                // Add the extra input value to the tx inputs value sum.
                tx_inputs_value_sum += txout.value.to_sat();
            }

            // Push the Liftup prevouts to the tx inputs.
            for (outpoint, txout) in lift_tx_inputs {
                tx_inputs.push((outpoint, txout.clone()));

                // Add the lift value to the tx inputs value sum.
                tx_inputs_value_sum += txout.value.to_sat();
            }
        }

        // Initialize the swapout tx outputs.
        let _swapout_tx_outputs = swapout_tx_outputs;

        let transaction_roughly_vbytesize = {
            // Rough virtual-size estimator:
            // - Input base: 40 vbytes per input (prevout+sequence+empty scriptsig framing).
            // - Payload input witness: rough script-path witness
            //   (sig + selector + tapscript(payload bytes) + control block).
            // - Lift input witness: rough script-path witness (sig + tapscript + control block).
            // - Output base: 43 vbytes per output.
            const TX_OVERHEAD_VBYTES: u64 = 11;
            const TXIN_BASE_VBYTES: u64 = 40;
            const TXOUT_BASE_VBYTES: u64 = 43;
            const PAYLOAD_WITNESS_BASE_ROUGH_VBYTES: u64 = 50;
            const LIFT_WITNESS_ROUGH_VBYTES: u64 = 41;

            let payload_script_bytes_len =
                prev_payload_tx_input.1.script_pubkey.as_bytes().len() as u64;
            let payload_witness_rough_vbytes =
                PAYLOAD_WITNESS_BASE_ROUGH_VBYTES + payload_script_bytes_len;
            let lift_inputs_count = lift_tx_inputs_len as u64;
            let tx_inputs_count = tx_inputs.len() as u64;

            let tx_outputs_count = {
                let mut n = 1u64; // change/payload output
                if new_projector_txout.is_some() {
                    n += 1;
                }
                n + _swapout_tx_outputs.len() as u64
            };

            TX_OVERHEAD_VBYTES
                + (tx_inputs_count * TXIN_BASE_VBYTES)
                + payload_witness_rough_vbytes
                + (lift_inputs_count * LIFT_WITNESS_ROUGH_VBYTES)
                + (tx_outputs_count * TXOUT_BASE_VBYTES)
        };
        println!(
            "transaction_roughly_vbytesize (estimated): {}",
            transaction_roughly_vbytesize
        );

        // Determine rough full transaction fee from feerate and estimated virtual bytesize.
        let bitcoin_transaction_fee = bitcoin_transaction_feerate
            .checked_mul(transaction_roughly_vbytesize)
            .ok_or(
                UnsignedBatchTxnConstructError::BitcoinTransactionFeeFromFeerateCheckedMulError,
            )?;
        println!(
            "bitcoin_transaction_fee (estimated): {}",
            bitcoin_transaction_fee
        );

        // Calculate the change value.
        let change_value = {
            // Initialize the change value to tx inputs value sum.
            let mut change_val = 0;

            // Add tx_inputs_value_sum
            change_val += tx_inputs_value_sum;

            // Add projector values to the change value.
            if let Some(projector_txout) = &new_projector_txout {
                change_val = change_val
                    .checked_sub(projector_txout.value.to_sat())
                    .ok_or(
                        UnsignedBatchTxnConstructError::ChangeValueProjectorValueCheckedSubError,
                    )?;
            }

            // Add swapout values to the change value.
            for tx_output in &_swapout_tx_outputs {
                change_val = change_val.checked_sub(tx_output.value.to_sat()).ok_or(
                    UnsignedBatchTxnConstructError::ChangeValueSwapoutValueCheckedSubError,
                )?;
            }

            // Minus the bitcoin transaction fee from the change value.
            change_val = change_val.checked_sub(bitcoin_transaction_fee).ok_or(
                UnsignedBatchTxnConstructError::ChangeValueBitcoinTransactionFeeCheckedSubError,
            )?;

            // Return the change value.
            change_val
        };

        // Fill transaction outputs.
        {
            // Push the new payload output to the tx outputs.
            tx_outputs.push(TxOut {
                value: bitcoin::Amount::from_sat(change_value),
                script_pubkey: new_payload_txout.script_pubkey,
            });

            // Push the new projector output to the tx outputs.
            if let Some(projector_txout) = new_projector_txout {
                tx_outputs.push(projector_txout);
            }

            // Extend the tx outputs with the swapout tx outputs.
            tx_outputs.extend(_swapout_tx_outputs);
        }

        // Construct the unsigned batch txn.
        let unsigned_batch_txn = UnsignedBatchTxn {
            tx_inputs,
            tx_outputs,
        };

        // Return the unsigned batch txn.
        Ok(unsigned_batch_txn)
    }

    /// BIP341 sha_prevouts.
    fn sha_prevouts(&self) -> [u8; 32] {
        let mut preimage = Vec::new();

        // Serialize each input outpoint as 32-byte txid || 4-byte vout (LE).
        for (outpoint, _) in &self.tx_inputs {
            preimage.extend(outpoint.bytes_36());
        }

        // Hash the preimage.
        sha256(&preimage)
    }

    /// BIP341 sha_amounts.
    fn sha_amounts(&self) -> [u8; 32] {
        let mut preimage = Vec::new();

        // Serialize each spent output amount as 8-byte little endian.
        for (_, tx_output) in &self.tx_inputs {
            preimage.extend(tx_output.value.to_sat().to_le_bytes());
        }

        // Hash the preimage.
        sha256(&preimage)
    }

    /// BIP341 sha_scriptpubkeys.
    fn sha_scriptpubkeys(&self) -> [u8; 32] {
        let mut preimage = Vec::new();

        // Serialize each spent output script as in CTxOut's script field.
        for (_, tx_output) in &self.tx_inputs {
            let scriptpubkey = tx_output.script_pubkey.as_bytes().to_vec();
            let scriptpubkey_len_varint = encode_varint(scriptpubkey.len() as u64);

            preimage.extend(scriptpubkey_len_varint);
            preimage.extend(scriptpubkey);
        }

        // Hash the preimage.
        sha256(&preimage)
    }

    /// BIP341 sha_sequences.
    fn sha_sequences(&self) -> [u8; 32] {
        let mut preimage = Vec::new();

        // UnsignedBatchTxn does not track per-input sequence, so serialize final sequence.
        for _ in &self.tx_inputs {
            preimage.extend(u32::MAX.to_le_bytes());
        }

        // Hash the preimage.
        sha256(&preimage)
    }

    /// BIP341 sha_outputs.
    fn sha_outputs(&self) -> [u8; 32] {
        let mut preimage = Vec::new();

        // Iterate over the tx outputs.
        for tx_output in &self.tx_outputs {
            let value = tx_output.value.to_sat();
            let value_le_bytes = value.to_le_bytes();
            preimage.extend(value_le_bytes);

            // Get the tx output scriptpubkey.
            let tx_output_scriptpubkey = tx_output.script_pubkey.as_bytes().to_vec();

            let tx_output_scriptpubkey_len = tx_output_scriptpubkey.len();
            let tx_output_scriptpubkey_len_varint =
                encode_varint(tx_output_scriptpubkey_len as u64);

            preimage.extend(tx_output_scriptpubkey_len_varint);

            preimage.extend(tx_output_scriptpubkey);
        }

        // Hash the preimage.
        sha256(&preimage)
    }

    /// BIP341/BIP342 Taproot sighash for the given input index.
    pub fn taproot_sighash(
        &self,
        input_index: u32,
        script_path_spend: Option<TapLeafHash>,
    ) -> Option<[u8; 32]> {
        let input_index_usize = usize::try_from(input_index).ok()?;
        if input_index_usize >= self.tx_inputs.len() {
            return None;
        }

        let hash_type = TAPROOT_SIGHASH_DEFAULT;
        let ext_flag = if script_path_spend.is_some() {
            1u8
        } else {
            0u8
        };

        let mut sigmsg = Vec::new();

        // Control byte.
        sigmsg.push(hash_type);

        // Common transaction data.
        sigmsg.extend(N_VERSION);
        sigmsg.extend(N_LOCKTIME);

        sigmsg.extend(self.sha_prevouts());
        sigmsg.extend(self.sha_amounts());
        sigmsg.extend(self.sha_scriptpubkeys());
        sigmsg.extend(self.sha_sequences());
        sigmsg.extend(self.sha_outputs());

        sigmsg.push(spend_type(ext_flag, false));

        sigmsg.extend(input_index.to_le_bytes());

        if let Some(tapleaf_hash) = script_path_spend {
            sigmsg.extend(tapleaf_hash);
            sigmsg.push(TAPSCRIPT_DEFAULT_PUBLIC_KEY_VERSION);
            sigmsg.extend(TAPSCRIPT_DEFAULT_CODESEPARATOR_POS.to_le_bytes());
        }

        let mut tap_sighash_preimage = Vec::new();
        tap_sighash_preimage.push(0x00); // epoch byte
        tap_sighash_preimage.extend(sigmsg);

        Some(tap_sighash_preimage.hash(Some(HashTag::TapSighash)))
    }
}

/// BIP341 spend_type = (ext_flag * 2) + annex_present.
///
/// `annex_present` should be `true` when annex exists, otherwise `false`.
pub fn spend_type(ext_flag: u8, annex_present: bool) -> u8 {
    (ext_flag * 2) + u8::from(annex_present)
}
