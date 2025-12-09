use crate::constructive::valtype::u8_ext::U8Ext;
use crate::constructive::valtype::val::atomic_val::ape::decode::error::decode_error::AtomicValAPEDecodeError;
use crate::constructive::valtype::val::atomic_val::atomic_val::AtomicVal;
use bit_vec::BitVec;

impl AtomicVal {
    /// Airly Payload Encoding (APE) decoding for `AtomicVal`.
    ///
    /// This function decodes an `AtomicVal` from an Airly Payload Encoding (APE) bit stream.
    /// The `AtomicVal` can be a u8, u16, u24, u32, u40, u48, u56, or u64.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `upper_bound` - The upper bound of the `AtomicVal`.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        upper_bound: u8,
    ) -> Result<AtomicVal, AtomicValAPEDecodeError> {
        // 1 Initialize a BitVec.
        let mut bits = BitVec::new();

        // 2 Determine the bitsize of the `AtomicVal`.
        let bitsize = AtomicVal::bitsize(upper_bound);

        // 3 Collect bitsize number of bits.
        for _ in 0..bitsize {
            bits.push(
                bit_stream
                    .next()
                    .ok_or(AtomicValAPEDecodeError::CollectBitsizeNumberBitsError)?,
            );
        }

        // 4 Convert the collected bits to a u8 value.
        let value =
            u8::from_bits(&bits, bitsize).ok_or(AtomicValAPEDecodeError::U8ExtFromBitsError)?;

        // 5 Convert the u8 value to an `AtomicVal`.
        let atomic_val = AtomicVal::new(value, upper_bound);

        // 6 Return the `AtomicVal`.
        Ok(atomic_val)
    }
}
