use crate::constructive::entry::ape::encode::error::encode_error::EntryAPEEncodeError;
use crate::constructive::entry::entry::Entry;
use bit_vec::BitVec;

impl Entry {
    /// Airly Payload Encoding (APE) encoding for `Entry`.
    ///
    /// This function encodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// # Arguments
    /// * `&self` - The `Entry` to encode.
    /// * `ops_price_base` - The base ops price of the `Entry`.
    /// * `encode_account_rank_as_longval` - Whether to encode the account rank as a `LongVal` or a `ShortVal`.
    /// * `encode_contract_rank_as_longval` - Whether to encode the contract rank as a `LongVal` or a `ShortVal`.
    pub async fn encode_ape(
        &self,
        ops_price_base: u32,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
    ) -> Result<BitVec, EntryAPEEncodeError> {
        // 1 Initialize the `Entry` APE bit vector.
        let mut bits = BitVec::new();

        // 2 Match on the `Entry` type.
        match self {
            // 2.a The `Entry` is a `Call`.
            Entry::Call(call) => {
                // 2.a.1 Encode the `Call` into an APE bit vector.
                let call_bits = call
                    .encode_ape(
                        ops_price_base,
                        encode_account_rank_as_longval,
                        encode_contract_rank_as_longval,
                    )
                    .await
                    .map_err(|e| EntryAPEEncodeError::CallAPEEncodeError(e))?;

                // 2.a.2 Extend the `Entry` APE bit vector with the `Call` APE bit vector.
                bits.extend(call_bits);
            }
        }

        // 3 Return the `Entry` APE bit vector.
        Ok(bits)
    }
}
