use crate::constructive::core_types::ops_price::ext::codec::ape::encode::error::encode_error::OpsPriceAPEEncodeError;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use bit_vec::BitVec;

impl OpsPrice {
    /// Encodes an `OpsPrice` into an Airly Payload Encoding (APE) bit vector.
    pub fn encode_ape(
        &self,
        base_ops_price: u32,
    ) -> Result<BitVec, OpsPriceAPEEncodeError> {
        if self.ops_price_ppm < base_ops_price {
            return Err(OpsPriceAPEEncodeError::OpsPriceBelowBaseOpsPrice {
                ops_price_ppm: self.ops_price_ppm,
                base_ops_price,
            });
        }

        let mut bits = BitVec::new();

        if self.ops_price_ppm == base_ops_price {
            bits.push(true);
        } else {
            let overhead = self.ops_price_ppm - base_ops_price;
            bits.push(false);
            bits.extend(ShortVal::new(overhead).encode_ape());
        }

        Ok(bits)
    }
}
