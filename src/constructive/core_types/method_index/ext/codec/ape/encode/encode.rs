use crate::constructive::core_types::method_index::ext::codec::ape::encode::error::encode_error::MethodIndexAPEEncodeError;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::valtypes::val::atomic_val::atomic_val::AtomicVal;
use bit_vec::BitVec;

const U8_METHOD_INDEX_UPPER_BOUND: usize = u8::MAX as usize;

impl MethodIndex {
    /// Encodes a `MethodIndex` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// When `methods_len` exceeds 255, the index is written as a raw u16 (16 little-endian
    /// bits). Otherwise it is encoded as an [`AtomicVal`] with `methods_len` as the upper bound.
    pub fn encode_ape(
        &self,
        methods_len: usize,
    ) -> Result<BitVec, MethodIndexAPEEncodeError> {
        if methods_len > U8_METHOD_INDEX_UPPER_BOUND {
            Ok(BitVec::from_bytes(&self.index.to_le_bytes()))
        } else {
            let upper_bound = methods_len as u8;
            let index = u8::try_from(self.index).map_err(|_| {
                MethodIndexAPEEncodeError::IndexDoesNotFitAtomicEncoding {
                    index: self.index,
                    methods_len,
                }
            })?;

            AtomicVal::new(index, upper_bound)
                .encode_ape()
                .map_err(MethodIndexAPEEncodeError::AtomicValAPEEncodeError)
        }
    }
}
