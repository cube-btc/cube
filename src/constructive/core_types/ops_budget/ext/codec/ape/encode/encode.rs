use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use bit_vec::BitVec;

impl OpsBudget {
    /// Encodes an `OpsBudget` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// When a budget is set, writes `true` followed by the value as a [`ShortVal`].
    /// When absent, writes a single `false` bit.
    pub fn encode_ape(&self) -> BitVec {
        let mut bits = BitVec::new();

        match self.ops_budget {
            Some(budget) => {
                bits.push(true);
                bits.extend(ShortVal::new(budget).encode_ape());
            }
            None => bits.push(false),
        }

        bits
    }
}
