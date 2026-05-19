use crate::constructive::core_types::ops_budget::ext::codec::ape::decode::error::decode_error::OpsBudgetAPEDecodeError;
use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;

impl OpsBudget {
    /// Decodes an `OpsBudget` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// A leading `true` bit means a [`ShortVal`] budget follows.
    /// A leading `false` bit means no budget is set.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<OpsBudget, OpsBudgetAPEDecodeError> {
        let is_set = bit_stream
            .next()
            .ok_or(OpsBudgetAPEDecodeError::UnexpectedEndOfBitstream)?;

        if !is_set {
            return Ok(OpsBudget::new(None));
        }

        let budget = ShortVal::decode_ape(bit_stream)
            .map_err(OpsBudgetAPEDecodeError::ShortValAPEDecodeError)?
            .value();

        Ok(OpsBudget::new(Some(budget)))
    }
}
