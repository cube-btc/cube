use crate::constructive::core_types::ops_price::ext::codec::ape::decode::error::decode_error::OpsPriceAPEDecodeError;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;

impl OpsPrice {
    /// Decodes an `OpsPrice` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// A leading `true` bit means `ops_price_ppm` equals `base_ops_price`.
    /// A leading `false` bit means a [`LongVal`] overhead follows, and the total is
    /// `base_ops_price + overhead`.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        base_ops_price: u32,
    ) -> Result<OpsPrice, OpsPriceAPEDecodeError> {
        let base_ops_price = u64::from(base_ops_price);

        let is_base = bit_stream
            .next()
            .ok_or(OpsPriceAPEDecodeError::UnexpectedEndOfBitstream)?;

        if is_base {
            return Ok(OpsPrice::new(base_ops_price));
        }

        let overhead = LongVal::decode_ape(bit_stream)
            .map_err(OpsPriceAPEDecodeError::LongValAPEDecodeError)?
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
