use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;

impl OpsBudget {
    /// Encodes this `OpsBudget` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: one presence byte (`0` = absent, `1` = present) followed by four
    /// little-endian bytes for the budget when present (zero-filled when absent).
    pub fn encode_sbe(&self) -> [u8; 5] {
        match self.ops_budget {
            Some(budget) => {
                let mut bytes = [0u8; 5];
                bytes[0] = 1;
                bytes[1..5].copy_from_slice(&budget.to_le_bytes());
                bytes
            }
            None => [0u8; 5],
        }
    }
}
