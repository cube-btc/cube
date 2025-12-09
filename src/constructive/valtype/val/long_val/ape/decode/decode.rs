use crate::constructive::valtype::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::valtype::val::long_val::long_val::{LongVal, LongValTier};
use bit_vec::BitVec;

/// Airly Payload Encoding (APE) decoding for `LongVal`.
impl LongVal {
    /// Decodes an `LongVal` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// This function decodes an `LongVal` from an Airly Payload Encoding (APE) bit stream.
    /// The `LongVal` can be a u8, u16, u24, u32, u40, u48, u56, or u64.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<LongVal, LongValAPEDecodeError> {
        // Decode the tier.
        let tier = match (bit_stream.next(), bit_stream.next(), bit_stream.next()) {
            // 000 for u8
            (Some(false), Some(false), Some(false)) => LongValTier::U8,
            // 001 for u16
            (Some(false), Some(false), Some(true)) => LongValTier::U16,
            // 010 for u24
            (Some(false), Some(true), Some(false)) => LongValTier::U24,
            // 011 for u32
            (Some(false), Some(true), Some(true)) => LongValTier::U32,
            // 100 for u40
            (Some(true), Some(false), Some(false)) => LongValTier::U40,
            // 101 for u48
            (Some(true), Some(false), Some(true)) => LongValTier::U48,
            // 110 for u56
            (Some(true), Some(true), Some(false)) => LongValTier::U56,
            // 111 for u64
            (Some(true), Some(true), Some(true)) => LongValTier::U64,
            _ => {
                return Err(LongValAPEDecodeError::TierBitsCollectError);
            }
        };

        // 2 Get the bit count for the tier.
        let bit_count = match tier {
            LongValTier::U8 => 8,
            LongValTier::U16 => 16,
            LongValTier::U24 => 24,
            LongValTier::U32 => 32,
            LongValTier::U40 => 40,
            LongValTier::U48 => 48,
            LongValTier::U56 => 56,
            LongValTier::U64 => 64,
        };

        // 3 Initialize the value bits.
        let mut value_bits = BitVec::new();

        // 4 Collect the value bits.
        for _ in 0..bit_count {
            value_bits.push(
                bit_stream
                    .next()
                    .ok_or(LongValAPEDecodeError::ValueBitsCollectError)?,
            );
        }

        // 5 Convert the value bits to bytes.
        let value_bytes = value_bits.to_bytes();

        // 6 Construct the long value.
        let long_val = LongVal::from_compact_bytes(&value_bytes)
            .ok_or(LongValAPEDecodeError::LongValFromCompactBytesConstructionError)?;

        // 7 Return the `LongVal`.
        Ok(long_val)
    }
}
