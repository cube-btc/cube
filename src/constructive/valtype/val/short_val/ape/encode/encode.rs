use crate::constructive::valtype::val::short_val::short_val::{ShortVal, ShortValTier};
use bit_vec::BitVec;

impl ShortVal {
    /// Encodes a `ShortVal` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes a `ShortVal` into an Airly Payload Encoding (APE) bit vector.
    /// The `ShortVal` can be a u8, u16, u24, or u32.
    ///
    /// # Arguments
    /// * `&self` - The `ShortVal` to encode.
    pub fn encode_ape(&self) -> BitVec {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Fill with tier bits.
        match self.uncommon_tier() {
            // 2.a 00 for u8
            ShortValTier::U8 => {
                bits.push(false);
                bits.push(false);
            }
            // 2.b 01 for u16
            ShortValTier::U16 => {
                bits.push(false);
                bits.push(true);
            }
            // 2.c 10 for u24
            ShortValTier::U24 => {
                bits.push(true);
                bits.push(false);
            }
            // 2.d 11 for u32
            ShortValTier::U32 => {
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
