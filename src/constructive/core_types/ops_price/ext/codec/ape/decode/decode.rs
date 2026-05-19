use crate::constructive::core_types::ops_price::ext::codec::ape::decode::error::decode_error::OpsPriceAPEDecodeError;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;

impl OpsPrice {
    /// Decodes an `OpsPrice` from an Airly Payload Encoding (APE) bit stream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        base_ops_price: u32,
    ) -> Result<OpsPrice, OpsPriceAPEDecodeError> {
        let is_base = bit_stream
            .next()
            .ok_or(OpsPriceAPEDecodeError::UnexpectedEndOfBitstream)?;

        if is_base {
            return Ok(OpsPrice::new(base_ops_price));
        }

        let overhead = ShortVal::decode_ape(bit_stream)
            .map_err(OpsPriceAPEDecodeError::ShortValAPEDecodeError)?
            .value();

        let ops_price_ppm = base_ops_price
            .checked_add(overhead)
            .ok_or(OpsPriceAPEDecodeError::OpsPriceTotalOverflow {
                base_ops_price,
                overhead,
            })?;

        Ok(OpsPrice::new(ops_price_ppm))
    }
}
