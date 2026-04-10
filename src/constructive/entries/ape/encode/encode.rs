use crate::constructive::entry::ape::encode::error::encode_error::EntryAPEEncodeError;
use crate::constructive::entry::entry::Entry;
use crate::inscriptive::registery::registery::REGISTERY;
use bit_vec::BitVec;

impl Entry {
    /// Airly Payload Encoding (APE) encoding for `Entry`.
    ///
    /// This function encodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
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
                        registery,
                        encode_account_rank_as_longval,
                        encode_contract_rank_as_longval,
                    )
                    .await
                    .map_err(|e| EntryAPEEncodeError::CallAPEEncodeError(e))?;

                // 2.a.3 Extend the `Entry` APE bit vector with the `Call` APE bit vector.
                bits.extend(call_bits);
            }

            // 2.b The `Entry` is a `Liftup`.
            Entry::Liftup(liftup) => {
                // 2.b.1 Push 1100 for the `Liftup` entry type.
                bits.push(true);
                bits.push(true);
                bits.push(false);
                bits.push(false);

                // 2.b.2 Encode the `Liftup`.
                let liftup_bits = liftup
                    .encode_ape(
                        execution_batch_height,
                        registery,
                        encode_account_rank_as_longval,
                    )
                    .await
                    .map_err(|e| EntryAPEEncodeError::LiftupAPEEncodeError(e))?;

                // 2.b.3 Extend the `Entry` APE bit vector with the `Liftup` APE bit vector.
                bits.extend(liftup_bits);
            }
        }

        // 3 Return the `Entry` APE bit vector.
        Ok(bits)
    }
}
