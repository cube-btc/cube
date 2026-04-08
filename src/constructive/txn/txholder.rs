use crate::{
    constructive::entry::entry_fields::valtypes::val::atomic_val::atomic_val::AtomicVal,
    inscriptive::utxo_set::utxo_set::UTXO_SET,
};
use bitcoin::{OutPoint, Transaction, TxOut};
use std::sync::Arc;

/// Default number of inputs: `Payload`.
const DEFAULT_NUM_INS: u32 = 1;

/// Default number of outputs: `Payload`, `VTXO Projector`, `Connector Projector`.
const DEFAULT_NUM_OUTS: u32 = 3;

/// Maximum 8 extra inputs are allowed.
const MAX_EXTRA_INS: u8 = 8;

/// A holder for a transaction, its extra inputs, and its input and output iterators.
pub struct TxHolder {
    utxo_set: UTXO_SET,
    /// The transaction.
    tx: Transaction,
    /// The number of extra inputs.
    extra_in: AtomicVal,
    /// The current input iterator.
    iterator_in: u32,
    /// The current output iterator.
    iterator_out: u32,
}

impl TxHolder {
    /// Creates a new `TxHolder`.
    pub fn new(utxo_set: &UTXO_SET, tx: Transaction, extra_in: u8) -> Option<TxHolder> {
        // Get initial iterator positions.
        let (iterator_in, iterator_out) = (
            // Default number of inputs plus the number of extra inputs (for expired projectors).
            DEFAULT_NUM_INS + extra_in as u32,
            // Default number of outputs. No extra outputs by design.
            DEFAULT_NUM_OUTS,
        );

        // Convert the extra inputs/outputs to their respective atomic values.
        let extra_in = AtomicVal::new(extra_in, MAX_EXTRA_INS);

        let tx_holder = TxHolder {
            utxo_set: Arc::clone(&utxo_set),
            tx,
            extra_in,
            iterator_in,
            iterator_out,
        };

        Some(tx_holder)
    }

    /// Returns the transaction.
    pub fn tx(&self) -> Transaction {
        self.tx.clone()
    }

    /// Returns the number of extra inputs.
    pub fn extra_in(&self) -> u8 {
        self.extra_in.value()
    }

    /// Returns the current input iterator position.
    pub fn input_iter_position(&self) -> u32 {
        self.iterator_in
    }

    /// Returns the current output iterator position.
    pub fn output_iter_position(&self) -> u32 {
        self.iterator_out
    }

    /// Iterates the input iterator by one.
    pub fn iterate_input(&mut self) {
        self.iterator_in += 1;
    }

    /// Iterates the output iterator by one.
    pub fn iterate_output(&mut self) {
        self.iterator_out += 1;
    }

    /// Returns whether both input and output iterators are at the end of the transaction.
    pub fn iterators_final(&self) -> bool {
        // Check if the input iterator is at the end of the transaction.
        if self.iterator_in != self.tx.input.len() as u32 {
            return false;
        }

        // Check if the output iterator is at the end of the transaction.
        if self.iterator_out != self.tx.output.len() as u32 {
            return false;
        }

        true
    }

    /// Returns the current input `OutPoint` and `TxOut`.
    pub async fn current_in(&self) -> Option<(OutPoint, TxOut)> {
        let current_iter = self.iterator_in as usize;
        let outpoint = self
            .tx
            .input
            .get(current_iter)
            .map(|input| input.previous_output)?;

        // Retrieve the TXOUT from the UTXO set.
        let txout = {
            let _utxo_set = self.utxo_set.lock().await;
            _utxo_set.txout_by_outpoint(&outpoint)?
        };

        // Return the outpoint and TXOUT.
        Some((outpoint, txout))
    }

    /// Returns the current output `TxOut`.
    pub fn current_out(&self) -> Option<TxOut> {
        let current_iter = self.iterator_out as usize;
        self.tx
            .output
            .get(current_iter)
            .map(|output| output.clone())
    }
}
