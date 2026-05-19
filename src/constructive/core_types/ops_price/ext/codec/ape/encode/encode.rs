use crate::constructive::core_types::ops_price::ext::codec::ape::encode::error::encode_error::OpsPriceAPEEncodeError;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use bit_vec::BitVec;

impl OpsPrice {
    /// Encodes an `OpsPrice` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// When `ops_price_ppm` equals `base_ops_price`, a single `true` bit is written.
    /// Otherwise a `false` bit is written followed by the overhead (`ops_price_ppm - base_ops_price`)
    /// as a [`LongVal`]. Encoding fails if `ops_price_ppm` is less than `base_ops_price`.
    pub fn encode_ape(
        &self,
        base_ops_price: u32,
    ) -> Result<BitVec, OpsPriceAPEEncodeError> {
        let base_ops_price = u64::from(base_ops_price);

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
            bits.extend(LongVal::new(overhead).encode_ape());
        }

        Ok(bits)
    }
}
