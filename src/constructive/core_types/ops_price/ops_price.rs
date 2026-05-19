use serde::{Deserialize, Serialize};

/// Parts-per-million ops price carried on an entry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpsPrice {
    pub ops_price_ppm: u32,
}

impl OpsPrice {
    /// Creates a new `OpsPrice`.
    pub fn new(ops_price_ppm: u32) -> OpsPrice {
        OpsPrice { ops_price_ppm }
    }
}