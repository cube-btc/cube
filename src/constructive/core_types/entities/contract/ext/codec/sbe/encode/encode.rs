use crate::constructive::entity::contract::contract::Contract;

type Bytes = Vec<u8>;

const CONTRACT_ID_SBE_LEN: usize = 32;
const REGISTERY_INDEX_SBE_LEN: usize = 8;

/// Fixed SBE payload size for [`Contract`].
pub const CONTRACT_SBE_LEN: usize = CONTRACT_ID_SBE_LEN + REGISTERY_INDEX_SBE_LEN;

impl Contract {
    /// Structural Byte-scope Encoding (SBE) for `Contract`.
    ///
    /// Layout: 32-byte `contract_id`, then little-endian `u64` `registery_index` (40 bytes total).
    pub fn encode_sbe(&self) -> Bytes {
        let mut bytes = Bytes::with_capacity(CONTRACT_SBE_LEN);
        bytes.extend_from_slice(&self.contract_id);
        bytes.extend_from_slice(&self.registery_index.to_le_bytes());
        bytes
    }
}
