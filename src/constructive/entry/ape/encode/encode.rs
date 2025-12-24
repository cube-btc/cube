use crate::constructive::entry::ape::encode::error::encode_error::EntryAPEEncodeError;
use crate::constructive::entry::entry::Entry;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;

impl Entry {
    /// Airly Payload Encoding (APE) encoding for `Entry`.
    ///
    /// This function encodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// # Arguments
    /// * `&self` - The `Entry` to encode.
    /// * `registery_manager` - The guarded `RegisteryManager` to get the `Account`'s rank value.
    /// * `encode_account_rank_as_longval` - Whether to encode the account rank as a `LongVal` or a `ShortVal`.
    /// * `encode_contract_rank_as_longval` - Whether to encode the contract rank as a `LongVal` or a `ShortVal`.
    pub async fn encode_ape(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
    ) -> Result<BitVec, EntryAPEEncodeError> {
        // 1 Initialize the `Entry` APE bit vector.
        let mut bits = BitVec::new();

        // 2 Match on the `Entry` type.
        match self {
            // 2.a The `Entry` is a `Call`.
            Entry::Call(call) => {
                // 2.a.1 Push 01 for the `Call` entry type.
                bits.push(false);
                bits.push(true);

                // 2.a.2 Encode the `Call`.
                let call_bits = call
                    .encode_ape(
                        registery_manager,
                        encode_account_rank_as_longval,
                        encode_contract_rank_as_longval,
                    )
                    .await
                    .map_err(|e| EntryAPEEncodeError::CallAPEEncodeError(e))?;

                // 2.a.3 Extend the `Entry` APE bit vector with the `Call` APE bit vector.
                bits.extend(call_bits);
            }
        }

        // 3 Return the `Entry` APE bit vector.
        Ok(bits)
    }
}
