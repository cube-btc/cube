use crate::constructive::core_types::method_index::ext::codec::ape::decode::error::decode_error::MethodIndexAPEDecodeError;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::valtypes::val::atomic_val::atomic_val::AtomicVal;

const U8_METHOD_INDEX_UPPER_BOUND: usize = u8::MAX as usize;

impl MethodIndex {
    /// Decodes a `MethodIndex` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// The wire format is chosen from `methods_len` (same rules as [`MethodIndex::encode_ape`]).
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        methods_len: usize,
    ) -> Result<MethodIndex, MethodIndexAPEDecodeError> {
        let index = if methods_len > U8_METHOD_INDEX_UPPER_BOUND {
            let mut value_bits = bit_vec::BitVec::new();
            for _ in 0..16 {
                value_bits.push(
                    bit_stream
                        .next()
                        .ok_or(MethodIndexAPEDecodeError::U16ValueBitsCollectError)?,
                );
            }

            let bytes = value_bits.to_bytes();
            u16::from_le_bytes([bytes[0], bytes[1]])
        } else {
            let upper_bound = methods_len as u8;
            AtomicVal::decode_ape(bit_stream, upper_bound)
                .map_err(MethodIndexAPEDecodeError::AtomicValAPEDecodeError)?
                .value() as u16
        };

        Ok(MethodIndex::new(index))
    }
}
