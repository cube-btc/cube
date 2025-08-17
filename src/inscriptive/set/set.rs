use super::{
    utxo_set::{UTXOSet, UTXO_SET},
    zktlc_set::{ZKTLCSet, ZKTLC_SET},
};
use crate::operative::Chain;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Guarded set for storing coins.
#[allow(non_camel_case_types)]
pub type COIN_SET = Arc<Mutex<CoinSet>>;

/// Set for storing bare and virtual coins (UTXOs and VTXOs).
pub struct CoinSet {
    utxo_set: UTXO_SET,
    zktlc_set: ZKTLC_SET,
}

impl CoinSet {
    /// Creates the CoinSet instance.
    pub fn new(chain: Chain) -> Option<COIN_SET> {
        // Construct utxo set.
        let utxo_set = UTXOSet::new(chain)?;

        // Construct zktlc set.
        let zktlc_set = ZKTLCSet::new(chain)?;

        let set = CoinSet { utxo_set, zktlc_set };

        Some(Arc::new(Mutex::new(set)))
    }

    /// Returns the UTXO set.
    pub fn utxo_set(&self) -> UTXO_SET {
        Arc::clone(&self.utxo_set)
    }

    /// Returns the ZKTLC set.
    pub fn zktlc_set(&self) -> ZKTLC_SET {
        Arc::clone(&self.zktlc_set)
    }
}
