use crate::constructive::bitcoiny::batch_txn::unsigned_batch_txn::error::construct_error::UnsignedBatchTxnConstructError;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut};

/// Represents an unsigned batch transaction.
#[derive(Debug, Clone)]
pub struct UnsignedBatchTxn {
    /// The Bitcoin transaction inputs of the batch.
    pub tx_inputs: Vec<OutPoint>,

    /// The Bitcoin transaction outputs of the batch.
    pub tx_outputs: Vec<TxOut>,
}

/// Represents an unsigned batch transaction constructor.
impl UnsignedBatchTxn {
    /// Constructs an unsigned batch transaction.
    pub fn construct(
        // Payload prevout
        prev_payload: (OutPoint, TxOut),
        // Extra inputs for expired projectors
        extra_ins: Vec<(OutPoint, TxOut)>,
        // Entry prevouts
        entries: Vec<Entry>,
        // Payload output
        new_payload: Payload,
        // Projector output
        new_projector: Option<Projector>,
        // Bitcoin transaction fee
        bitcoin_transaction_fee: u64,
    ) -> Result<UnsignedBatchTxn, UnsignedBatchTxnConstructError> {
        // Initialzie the tx inputs.
        let mut tx_inputs = Vec::new();

        // Initialzie the tx outputs.
        let mut tx_outputs = Vec::new();

        // Initialize the tx inputs value sum.
        let mut tx_inputs_value_sum = 0;

        // Fill transaction inputs.
        {
            // Push the prev payload outpoint to the tx inputs.
            tx_inputs.push(prev_payload.0);

            // Add the prev payload value to the tx inputs value sum.
            tx_inputs_value_sum += prev_payload.1.value.to_sat();

            // Push the extra inputs to the tx inputs.
            for (outpoint, txout) in extra_ins {
                tx_inputs.push(outpoint);

                // Add the extra input value to the tx inputs value sum.
                tx_inputs_value_sum += txout.value.to_sat();
            }

            // Push the Liftup prevouts to the tx inputs.
            for entry in &entries {
                if let Entry::Liftup(liftup) = entry {
                    for lift in &liftup.lift_tx_inputs {
                        tx_inputs.push(lift.outpoint());

                        // Add the lift value to the tx inputs value sum.
                        tx_inputs_value_sum += lift.lift_value_in_satoshis();
                    }
                }
            }
        }

        // Initialize the swapout tx outputs.
        let mut swapout_tx_outputs = Vec::<TxOut>::new();

        // Fill swapout tx outputs.
        for entry in &entries {
            //if let Entry::Swapout(swapout) = entry {
            //    swapout_tx_outputs.push(swapout.txout());
            //}
        }

        // Calculate the change value.
        let change_value = {
            // Initialize the change value to tx inputs value sum.
            let mut change_value = tx_inputs_value_sum;

            // Add projector values to the change value.
            if let Some(projector) = &new_projector {
                change_value.checked_sub(projector.satoshi_amount).ok_or(
                    UnsignedBatchTxnConstructError::ChangeValueProjectorValueCheckedSubError,
                )?;
            }

            // Add swapout values to the change value.
            for tx_output in &swapout_tx_outputs {
                change_value.checked_sub(tx_output.value.to_sat()).ok_or(
                    UnsignedBatchTxnConstructError::ChangeValueSwapoutValueCheckedSubError,
                )?;
            }

            // Minus the bitcoin transaction fee from the change value.
            change_value.checked_sub(bitcoin_transaction_fee).ok_or(
                UnsignedBatchTxnConstructError::ChangeValueBitcoinTransactionFeeCheckedSubError,
            )?;

            // Return the change value.
            change_value
        };

        // Fill transaction outputs.
        {
            // Push the new payload output to the tx outputs.
            {
                // Get the new payload scriptpubkey.
                let new_payload_scriptpubkey = new_payload
                    .scriptpubkey()
                    .ok_or(UnsignedBatchTxnConstructError::NewPayloadScriptpubkeyError)?;

                // Construct the payload txout.
                let new_payload_txout = TxOut {
                    value: Amount::from_sat(change_value),
                    script_pubkey: ScriptBuf::from(new_payload_scriptpubkey),
                };

                // Push the payload txout to the tx outputs.
                tx_outputs.push(new_payload_txout);
            }

            // Push the new projector output to the tx outputs.
            if let Some(projector) = &new_projector {
                // Get the projector scriptpubkey.
                let new_projector_scriptpubkey = projector.scriptpubkey.clone();

                // Construct the projector txout.
                let new_projector_txout = TxOut {
                    value: Amount::from_sat(projector.satoshi_amount),
                    script_pubkey: ScriptBuf::from(new_projector_scriptpubkey),
                };

                // Push the projector txout to the tx outputs.
                tx_outputs.push(new_projector_txout);
            }

            // Extend the tx outputs with the swapout tx outputs.
            tx_outputs.extend(swapout_tx_outputs);
        }

        // Construct the unsigned batch txn.
        let unsigned_batch_txn = UnsignedBatchTxn {
            tx_inputs,
            tx_outputs,
        };

        // Return the unsigned batch txn.
        Ok(unsigned_batch_txn)
    }
}
