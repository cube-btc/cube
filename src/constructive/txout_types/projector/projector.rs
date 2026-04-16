use serde::Deserialize;
use serde::Serialize;
use bitcoin::{OutPoint, TxOut};

#[derive(Clone, Serialize, Deserialize)]
pub struct Projector {
    pub scriptpubkey: Vec<u8>,
    pub satoshi_amount: u64,
    pub location: Option<(OutPoint, TxOut)>,
}

impl Projector {
    /// Returns the location of the Projector.
    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        self.location.clone()
    }
}