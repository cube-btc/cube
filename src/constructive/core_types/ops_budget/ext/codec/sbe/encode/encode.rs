use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::valtypes::val::short_val::short_val::{ShortVal, ShortValTier};

type Bytes = Vec<u8>;

fn short_val_tier_byte(tier: ShortValTier) -> u8 {
    match tier {
        ShortValTier::U8 => 0,
        ShortValTier::U16 => 1,
        ShortValTier::U24 => 2,
        ShortValTier::U32 => 3,
    }
}

impl OpsBudget {
    /// Encodes this `OpsBudget` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: presence byte (`0` = absent, `1` = present). When present, a tier byte
    /// (`0`–`3`) plus 1–4 little-endian compact bytes for the budget value.
    pub fn encode_sbe(&self) -> Bytes {
        // 1 Match on whether a budget is set.
        match self.ops_budget {
            None => vec![0x00],
            Some(budget) => {
                // 2 Encode the budget as a compact `ShortVal`.
                let short_val = ShortVal::new(budget);
                let mut bytes = Bytes::new();
                bytes.push(0x01);
                bytes.push(short_val_tier_byte(short_val.uncommon_tier()));
                bytes.extend_from_slice(&short_val.compact_bytes());

                // 3 Return bytes.
                bytes
            }
        }
    }
}
