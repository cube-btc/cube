use crate::constructive::valtype::val::long_val::long_val::{LongVal, LongValTier};
use bit_vec::BitVec;

/// Airly Payload Encoding (APE) encoding for `LongVal`.
impl LongVal {
    /// Encodes a `LongVal` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes a `LongVal` into an Airly Payload Encoding (APE) bit vector.
    /// The `LongVal` can be a u8, u16, u24, u32, u40, u48, u56, or u64.
    ///
    /// # Arguments
    /// * `&self` - The `LongVal` to encode.
    pub fn encode_ape(&self) -> BitVec {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Fill with tier bits.
        match self.tier() {
            // 2.a 000 for u8
            LongValTier::U8 => {
                bits.push(false);
                bits.push(false);
                bits.push(false);
            }
            // 2.b 001 for u16
            LongValTier::U16 => {
                bits.push(false);
                bits.push(false);
                bits.push(true);
            }
            // 2.c 010 for u24
            LongValTier::U24 => {
                bits.push(false);
                bits.push(true);
                bits.push(false);
            }
            // 2.d 011 for u32
            LongValTier::U32 => {
                bits.push(false);
                bits.push(true);
                bits.push(true);
            }
            // 2.e 100 for u40
            LongValTier::U40 => {
                bits.push(true);
                bits.push(false);
                bits.push(false);
            }
            // 2.f 101 for u48
            LongValTier::U48 => {
                bits.push(true);
                bits.push(false);
                bits.push(true);
            }

            // 2.g 110 for u56
            LongValTier::U56 => {
                bits.push(true);
                bits.push(true);
                bits.push(false);
            }

            // 2.h 111 for u64
            LongValTier::U64 => {
                bits.push(true);
                bits.push(true);
                bits.push(true);
            }
        }

        // 3 Convert the compact bytes to bits.
        let value_bits = BitVec::from_bytes(&self.compact_bytes());

        // 4 Extend the bits with the value bits.
        bits.extend(value_bits);

        // 5 Return the bit vector.
        bits
    }
}
