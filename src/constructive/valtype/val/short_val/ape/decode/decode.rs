use crate::constructive::valtype::val::short_val::{
    ape::decode::error::decode_error::ShortValAPEDecodeError,
    short_val::{ShortVal, ShortValTier},
};
use bit_vec::BitVec;

/// Airly Payload Encoding (APE) decoding for `ShortVal`.
impl ShortVal {
    /// Decodes a `ShortVal` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// This function decodes a `ShortVal` from an Airly Payload Encoding (APE) bit stream.
    /// The `ShortVal` can be a u8, u16, u24, or u32.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<ShortVal, ShortValAPEDecodeError> {
        // 1 Decode the  tier.
        let tier = match (bit_stream.next(), bit_stream.next()) {
            (Some(false), Some(false)) => ShortValTier::U8,
            (Some(false), Some(true)) => ShortValTier::U16,
            (Some(true), Some(false)) => ShortValTier::U24,
            (Some(true), Some(true)) => ShortValTier::U32,
            _ => {
                return Err(ShortValAPEDecodeError::TierBitsCollectError);
            }
        };

        // 2 Get the bit count for the tier.
        let bit_count = match tier {
            ShortValTier::U8 => 8,
            ShortValTier::U16 => 16,
            ShortValTier::U24 => 24,
            ShortValTier::U32 => 32,
        };

        // 3 Initialize the value bits.
        let mut value_bits = BitVec::new();

        // 4 Collect the value bits.
        for _ in 0..bit_count {
            value_bits.push(
                bit_stream
                    .next()
                    .ok_or(ShortValAPEDecodeError::ValueBitsCollectError)?,
            );
        }

        // 5 Convert the value bits to bytes.
        let value_bytes = value_bits.to_bytes();

        // 6 Construct the short value.
        let short_val = ShortVal::from_compact_bytes(&value_bytes)
            .ok_or(ShortValAPEDecodeError::ShortValFromCompactBytesConstructionError)?;

        // 7 Return the `ShortVal`.
        Ok(short_val)
    }
}
