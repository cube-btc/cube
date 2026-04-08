use crate::constructive::entry::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::constructive::entry::entry::Entry;
use crate::constructive::entry::entry_types::call::call::Call;
use crate::inscriptive::registery::registery::REGISTERY;

impl Entry {
    /// Decodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    /// The `Entry` can be a `Call` or a `Move`.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `base_ops_price` - The base ops price of the `Entry`.
    /// * `registery` - The `Registery`.
    /// * `decode_account_rank_as_longval` - Whether to decode the account rank as a `LongVal` or a `ShortVal`.
    /// * `decode_contract_rank_as_longval` - Whether to decode the contract rank as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        base_ops_price: u32,
        decode_account_rank_as_longval: bool,
        decode_contract_rank_as_longval: bool,
        registery: &REGISTERY,
    ) -> Result<Entry, EntryAPEDecodeError> {
        // 1 Collect one bit to determine if the `Entry` is from the `Common Branch` or the `Uncommon Branch`.
        let common_or_uncommon_branch_bit = bit_stream
            .next()
            .ok_or(EntryAPEDecodeError::CommonUncommonBranchBitCollectError)?;

        // 2 Match on whether the `Entry` is from the `Common Branch` or the `Uncommon Branch`.
        let entry: Entry = match common_or_uncommon_branch_bit {
            // 2.a The `Entry` is from the `Common Branch`.
            true => {
                // 2.a.1 Collect one bit to determine if the `Entry` is a `Move` or a `Call`.
                let move_or_call_bit = bit_stream
                    .next()
                    .ok_or(EntryAPEDecodeError::MoveOrCallBitCollectError)?;

                // 2.a.2 Match on whether the `Entry` is a `Move` or a `Call`.
                match move_or_call_bit {
                    // 2.a.2.a The `Entry` is a `Move`.
                    true => panic!("Move is not implemented yet."),

                    // 2.a.2.b The `Entry` is a `Call`.
                    false => {
                        // 2.a.2.b.1 Decode the `Call` entry.
                        let call_entry: Call = Call::decode_ape(
                            bit_stream,
                            base_ops_price,
                            decode_account_rank_as_longval,
                            decode_contract_rank_as_longval,
                            registery,
                        )
                        .await
                        .map_err(|e| EntryAPEDecodeError::CallEntryAPEDecodeError(e))?;

                        // 2.a.2.b.2 Return the `Call` `Entry`.
                        Entry::Call(call_entry)
                    }
                }
            }
            // 2.b The `Entry` is from the `Uncommon Branch`.
            false => panic!("Uncommon Branch is not implemented yet."),
        };

        // 3 Return the `Entry`.
        Ok(entry)
    }
}
