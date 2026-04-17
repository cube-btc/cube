use crate::constructive::bitcoiny::batch_txn::{
    signed_batch_txn::error::construct_error::SignedBatchTxnConstructError,
    unsigned_batch_txn::error::construct_error::UnsignedBatchTxnConstructError,
    unsigned_batch_txn::unsigned_batch_txn::UnsignedBatchTxn,
};
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::txout_types::lift::lift::Lift;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::transmutative::codec::varint::encode_varint;
use crate::transmutative::hash::sha256;
use crate::transmutative::key::KeyHolder;
use crate::transmutative::secp::schnorr::{self, SchnorrSigningMode};
use bitcoin::hashes::Hash;
use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};
use serde::{Deserialize, Serialize};

// Bare transaction fields:
const N_VERSION: [u8; 4] = [0x02, 0x00, 0x00, 0x00];
const N_LOCKTIME: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
const N_SEQUENCE_MAX: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

/// BIP141 witness serialization marker and flag.
const SEGWIT_MARKER: u8 = 0x00;
const SEGWIT_FLAG: u8 = 0x01;

type Bytes = Vec<u8>;

type Witness = Vec<Bytes>;

/// Represents a signed batch transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBatchTxn {
    /// The Bitcoin transaction inputs of the batch.
    pub tx_inputs: Vec<(OutPoint, TxOut, Witness)>,

    /// The Bitcoin transaction outputs of the batch.
    pub tx_outputs: Vec<TxOut>,
}

impl SignedBatchTxn {
    /// Constructs a signed batch transaction.
    pub fn construct(
        // Tx inputs
        prev_payload: Payload,
        prev_projectors: Vec<Projector>,
        // Entries
        entries: Vec<Entry>,
        // Tx outputs
        new_payload: Payload,
        new_projector: Option<Projector>,
        // Tx fee
        bitcoin_transaction_fee: u64,
        // Engine key
        engine_keyholder: &KeyHolder,
    ) -> Result<SignedBatchTxn, SignedBatchTxnConstructError> {
        // Prev projectors are not supported for the time being
        {
            if prev_projectors.len() != 0 {
                return Err(SignedBatchTxnConstructError::PrevProjectorsNotSupportedError);
            }
        };

        let prev_payload_tx_input: (OutPoint, TxOut) = match prev_payload.location() {
            Some((outpoint, txout)) => (outpoint, txout),
            None => return Err(SignedBatchTxnConstructError::PayloadLocationNotFoundError),
        };

        let projector_tx_inputs: Vec<(OutPoint, TxOut)> = prev_projectors
            .iter()
            .map(|projector| {
                projector
                    .location
                    .as_ref()
                    .map(|(outpoint, txout)| (outpoint.clone(), txout.clone()))
                    .ok_or(SignedBatchTxnConstructError::ProjectorLocationNotFoundError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let lift_tx_inputs: Vec<(OutPoint, TxOut)> = {
            let mut lift_tx_inputs = Vec::new();
            for entry in &entries {
                if let Entry::Liftup(liftup) = entry {
                    for lift in &liftup.lift_tx_inputs {
                        lift_tx_inputs.push((lift.outpoint(), lift.txout()));
                    }
                }
            }
            lift_tx_inputs
        };

        let new_payload_scriptpubkey = new_payload.calculated_scriptpubkey().ok_or(
            SignedBatchTxnConstructError::UnsignedBatchTxnConstructError(
                UnsignedBatchTxnConstructError::NewPayloadScriptpubkeyError,
            ),
        )?;

        let new_payload_txout = TxOut {
            value: Amount::from_sat(0),
            script_pubkey: ScriptBuf::from(new_payload_scriptpubkey),
        };

        let new_projector_txout = new_projector.map(|projector| TxOut {
            value: Amount::from_sat(projector.satoshi_amount),
            script_pubkey: ScriptBuf::from(projector.scriptpubkey),
        });

        let unsigned_batch_txn = UnsignedBatchTxn::construct(
            prev_payload_tx_input,
            projector_tx_inputs,
            lift_tx_inputs,
            new_payload_txout,
            new_projector_txout,
            bitcoin_transaction_fee,
        )
        .map_err(SignedBatchTxnConstructError::UnsignedBatchTxnConstructError)?;

        // Initialize the tx input witnesses.
        let mut tx_input_witnesses = Vec::<Witness>::new();

        let mut tx_input_index_iterator = 0;

        // Fill tx input witnesses

        // Fill prev payload witness
        {
            let (prev_payload_tapleaf_hash, prev_payload_tapscript, prev_payload_control_block) =
                prev_payload.p2tr_script_path_spend_elements();

            let prev_payload_taproot_sighash: [u8; 32] = unsigned_batch_txn
                .taproot_sighash(tx_input_index_iterator, Some(prev_payload_tapleaf_hash))
                .ok_or(SignedBatchTxnConstructError::PrevPayloadTaprootSighashConstructionError)?;

            let prev_payload_taproot_signature: [u8; 64] = schnorr::sign(
                engine_keyholder.secp_secret_key_bytes(),
                prev_payload_taproot_sighash,
                SchnorrSigningMode::BIP340,
            )
            .ok_or(SignedBatchTxnConstructError::PrevPayloadTaprootSignError)?;

            // BIP342 script-path witness stack:
            // <sig> <engine-branch selector=1> <tapscript> <control block>
            let prev_payload_witness: Vec<Bytes> = vec![
                prev_payload_taproot_signature.to_vec(),
                vec![0x01],
                prev_payload_tapscript,
                prev_payload_control_block,
            ];

            tx_input_witnesses.push(prev_payload_witness);
            tx_input_index_iterator += 1;
        }

        // Fill prev projectors witnesses
        {
            // Not supported for the time being.
        }

        // Fill LiftV1 witnesses
        {
            for entry in &entries {
                if let Entry::Liftup(liftup) = entry {
                    for lift in &liftup.lift_tx_inputs {
                        // Error if its Liftv2
                        match lift {
                            Lift::LiftV1(liftv1) => {
                                // Get the tapleaf hash, tapscript, and control block
                                let (
                                    prev_liftv1_tapleaf_hash,
                                    prev_liftv1_tapscript,
                                    prev_liftv1_control_block,
                                ) = liftv1.p2tr_script_path_spend_elements();

                                // Get the taproot sighash
                                let prev_liftv1_taproot_sighash: [u8; 32] = unsigned_batch_txn
                                    .taproot_sighash(tx_input_index_iterator, Some(prev_liftv1_tapleaf_hash))
                                    .ok_or(SignedBatchTxnConstructError::PrevLiftV1TaprootSighashConstructionError(liftv1.clone()))?;

                                // Get the taproot signature
                                let prev_liftv1_taproot_signature: [u8; 64] = schnorr::sign(
                                    engine_keyholder.secp_secret_key_bytes(),
                                    prev_liftv1_taproot_sighash,
                                    SchnorrSigningMode::BIP340,
                                )
                                .ok_or(SignedBatchTxnConstructError::PrevLiftV1TaprootSignError(
                                    liftv1.clone(),
                                ))?;

                                let prev_liftv1_witness = vec![
                                    prev_liftv1_taproot_signature.to_vec(),
                                    prev_liftv1_tapscript,
                                    prev_liftv1_control_block,
                                ];

                                tx_input_witnesses.push(prev_liftv1_witness);
                            }
                            Lift::LiftV2(liftv2) => {
                                return Err(SignedBatchTxnConstructError::LiftV2NotSupportedError(
                                    liftv2.clone(),
                                ));
                            }
                            Lift::Unknown { .. } => {
                                return Err(
                                    SignedBatchTxnConstructError::UnknownLiftNotSupportedError,
                                );
                            }
                        }
                    }
                }
            }
        }

        let tx_inputs: Vec<(OutPoint, TxOut, Witness)> = unsigned_batch_txn
            .tx_inputs
            .into_iter()
            .zip(tx_input_witnesses.into_iter())
            .map(|((outpoint, txout), witness)| (outpoint, txout, witness))
            .collect();

        Ok(SignedBatchTxn {
            tx_inputs,
            tx_outputs: unsigned_batch_txn.tx_outputs,
        })
    }

    /// Returns the transaction input outpoints.
    pub fn tx_input_outpoints(&self) -> Vec<OutPoint> {
        self.tx_inputs
            .iter()
            .map(|(outpoint, _, _)| outpoint.clone())
            .collect()
    }

    /// Returns the transaction outputs.
    pub fn tx_outputs(&self) -> Vec<TxOut> {
        self.tx_outputs.clone()
    }

    /// Serializes the Bitcoin transaction.
    pub fn serialize_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&N_VERSION);
        buf.push(SEGWIT_MARKER);
        buf.push(SEGWIT_FLAG);

        buf.extend_from_slice(&encode_varint(self.tx_inputs.len() as u64));
        for (outpoint, _, _) in &self.tx_inputs {
            push_legacy_txin(&mut buf, outpoint);
        }

        buf.extend_from_slice(&encode_varint(self.tx_outputs.len() as u64));
        for txout in &self.tx_outputs {
            push_txout(&mut buf, txout);
        }

        for (_, _, w) in &self.tx_inputs {
            push_witness(&mut buf, w);
        }

        buf.extend_from_slice(&N_LOCKTIME);
        buf
    }

    /// Serializes the transaction for txid.
    pub fn serialize_bytes_for_txid(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&N_VERSION);

        buf.extend_from_slice(&encode_varint(self.tx_inputs.len() as u64));
        for (outpoint, _, _) in &self.tx_inputs {
            push_legacy_txin(&mut buf, outpoint);
        }

        buf.extend_from_slice(&encode_varint(self.tx_outputs.len() as u64));
        for txout in &self.tx_outputs {
            push_txout(&mut buf, txout);
        }

        buf.extend_from_slice(&N_LOCKTIME);
        buf
    }

    /// Returns the transaction id.
    pub fn txid(&self) -> Txid {
        let preimage = self.serialize_bytes_for_txid();
        let first = sha256(&preimage);
        Txid::from_byte_array(sha256(&first))
    }
}

fn push_outpoint(buf: &mut Vec<u8>, outpoint: &OutPoint) {
    buf.extend_from_slice(&outpoint.txid.to_byte_array());
    buf.extend_from_slice(&outpoint.vout.to_le_bytes());
}

fn push_legacy_txin(buf: &mut Vec<u8>, outpoint: &OutPoint) {
    push_outpoint(buf, outpoint);
    buf.push(0x00); // empty scriptSig
    buf.extend_from_slice(&N_SEQUENCE_MAX);
}

fn push_txout(buf: &mut Vec<u8>, txout: &TxOut) {
    buf.extend_from_slice(&txout.value.to_sat().to_le_bytes());
    let spk = txout.script_pubkey.as_bytes();
    buf.extend_from_slice(&encode_varint(spk.len() as u64));
    buf.extend_from_slice(spk);
}

/// One input’s witness stack (BIP141): stack item count, then each item length-prefixed.
fn push_witness(buf: &mut Vec<u8>, witness: &Witness) {
    buf.extend_from_slice(&encode_varint(witness.len() as u64));
    for item in witness {
        buf.extend_from_slice(&encode_varint(item.len() as u64));
        buf.extend_from_slice(item);
    }
}
