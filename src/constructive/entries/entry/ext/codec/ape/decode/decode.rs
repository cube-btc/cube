use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry::ext::codec::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use bitcoin::OutPoint;

impl Entry {
    /// Decodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes an `Entry` as an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        engine_key: [u8; 32],
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        tx_inputs_iter: &mut impl Iterator<Item = OutPoint>,
        decode_account_rank_as_longval: bool,
        decode_contract_rank_as_longval: bool,
        base_ops_price: u32,
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
    ) -> Result<Entry, EntryAPEDecodeError> {
        // 1 Collect one bit to determine if the `Entry` is from the `Common Branch` or the `Uncommon Branch`.
        let common_or_uncommon_branch_bit = bit_stream
            .next()
            .ok_or(EntryAPEDecodeError::CommonUncommonBranchBitCollectError)?;

        // 2 Match on whether the `Entry` is from the `Common Branch` or the `Uncommon Branch`.
        let entry: Entry = match common_or_uncommon_branch_bit {
            // 2.a The `Entry` is from the `Common Branch`.
            false => {
                // 2.a.1 Collect one bit to determine if the `Entry` is a `Move` or a `Call`.
                let move_or_call_bit = bit_stream
                    .next()
                    .ok_or(EntryAPEDecodeError::MoveOrCallBitCollectError)?;

                // 2.a.2 Match on whether the `Entry` is a `Move` or a `Call`.
                match move_or_call_bit {
                    // 2.a.2.a The `Entry` is a `Move`.
                    false => {
                        // 2.a.2.a.1 Decode the `Move` entry.
                        let move_entry: Move = Move::decode_ape(
                            execution_batch_height,
                            bit_stream,
                            decode_account_rank_as_longval,
                            registery,
                        )
                        .await
                        .map_err(EntryAPEDecodeError::MoveEntryAPEDecodeError)?;

                        // 2.a.2.a.2 Return the `Move` `Entry`.
                        Entry::Move(move_entry)
                    }

                    // 2.a.2.b The `Entry` is a `Call`.
                    true => {
                        // 2.a.2.b.1 Decode the `Call` entry.
                        let call_entry: Call = Call::decode_ape(
                            bit_stream,
                            base_ops_price,
                            decode_account_rank_as_longval,
                            decode_contract_rank_as_longval,
                            registery,
                        )
                        .await
                        .map_err(EntryAPEDecodeError::CallEntryAPEDecodeError)?;

                        // 2.a.2.b.2 Return the `Call` `Entry`.
                        Entry::Call(call_entry)
                    }
                }
            }

            // 2.b The `Entry` is from the `Uncommon Branch`.
            true => {
                // 2.b.1 Collect one bit to determine if the `Entry` is in the Liquidity Branch or the Outer Branch.
                let liquidity_or_outer_branch_bit = bit_stream
                    .next()
                    .ok_or(EntryAPEDecodeError::LiquidityOrOuterBranchBitCollectError)?;

                // 2.b.2 Match on whether the `Entry` is in the Liquidity Branch or the Outer Branch.
                match liquidity_or_outer_branch_bit {
                    // 2.b.2.a The `Entry` is in the Liquidity Branch.
                    false => {
                        // 2.b.2.a.1 Collect one bit to determine if the `Entry` is `Add` or `Sub`.
                        let add_or_sub_bit = bit_stream
                            .next()
                            .ok_or(EntryAPEDecodeError::AddOrSubBitCollectError)?;

                        // 2.b.2.a.2 Match on whether the `Entry` is a `Add` or a `Sub`.
                        match add_or_sub_bit {
                            // 2.b.2.a.2.a The `Entry` is a `Add`.
                            false => panic!("Add is not implemented yet."),

                            // 2.b.2.a.2.b The `Entry` is a `Sub`.
                            true => panic!("Sub is not implemented yet."),
                        }
                    }

                    // 2.b.2.b The `Entry` is in the Outer Branch.
                    true => {
                        // 2.b.2.b.1 Collect one bit to determine if the `Entry` is in the `Gateway Branch` or the `Outer Right Branch`.
                        let gateway_or_outer_right_branch_bit = bit_stream
                            .next()
                            .ok_or(EntryAPEDecodeError::GatewayOrOuterRightBranchBitCollectError)?;

                        // 2.b.2.b.1 Match on whether the `Entry` is in the Gateway Branch or the Outer Right Branch.
                        match gateway_or_outer_right_branch_bit {
                            // 2.b.2.b.1.a The `Entry` is in the Gateway Branch.
                            false => {
                                // 2.b.2.b.1.a.1 Collect one bit to determine if the `Entry` is a `Liftup` or a `Swapout`.
                                let liftup_or_swapout_bit = bit_stream
                                    .next()
                                    .ok_or(EntryAPEDecodeError::LiftupOrSwapoutBitCollectError)?;

                                // 2.b.2.b.1.a.1 Match on whether the `Entry` is a `Liftup` or a `Swapout`.
                                match liftup_or_swapout_bit {
                                    // 2.b.2.b.1.a.1.a The `Entry` is a `Liftup`.
                                    false => {
                                        // 2.b.2.b.1.a.1.a.1 Decode the `Liftup` entry.
                                        let liftup_entry: Liftup = Liftup::decode_ape(
                                            engine_key,
                                            execution_batch_height,
                                            bit_stream,
                                            tx_inputs_iter,
                                            decode_account_rank_as_longval,
                                            utxo_set,
                                            registery,
                                        )
                                        .await
                                        .map_err(EntryAPEDecodeError::LiftupEntryAPEDecodeError)?;

                                        // 2.b.2.b.1.a.1.a.2 Return the `Liftup` `Entry`.
                                        Entry::Liftup(liftup_entry)
                                    }

                                    // 2.b.2.b.1.a.1.b The `Entry` is a `Swapout`.
                                    true => panic!("Swapout is not implemented yet."),
                                }
                            }

                            // 2.b.2.b.1.b The `Entry` is in the Outer Right Branch.
                            true => {
                                // Collect one bit to determine if the `Entry` is in the `Outer Lowermost Branch` or the `Reserved Branch`.
                                let outer_lowermost_or_reserved_branch_bit =
                                    bit_stream.next().ok_or(
                                        EntryAPEDecodeError::OuterLowermostOrReservedBranchBitCollectError,
                                    )?;

                                // 2.b.2.b.1.b.1 Match on whether the `Entry` is in the `Outer Lowermost Branch` or the `Reserved Branch`.
                                match outer_lowermost_or_reserved_branch_bit {
                                    // 2.b.2.b.1.b.1.a The `Entry` is in the `Outer Lowermost Branch`.
                                    false => {
                                        // 2.b.2.b.1.b.1.a.1 Collect one bit to determine if the `Entry` is a `Deploy` or a `Config`.
                                        let deploy_or_config_bit = bit_stream.next().ok_or(
                                            EntryAPEDecodeError::DeployOrConfigBitCollectError,
                                        )?;

                                        // 2.b.2.b.1.b.1.a.2 Match on whether the `Entry` is a `Deploy` or a `Config`.
                                        match deploy_or_config_bit {
                                            // 2.b.2.b.1.b.1.a.2.a The `Entry` is a `Deploy`.
                                            false => panic!("Deploy is not implemented yet."),

                                            // 2.b.2.b.1.b.1.a.2.b The `Entry` is a `Config`.
                                            true => panic!("Config is not implemented yet."),
                                        }
                                    }

                                    // 2.b.2.b.1.b.1.b The `Entry` is in the `Reserved Branch`.
                                    true => {
                                        return Err(
                                            EntryAPEDecodeError::ReservedBranchEncounteredError,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        // 3 Return the decoded `Entry`.
        Ok(entry)
    }
}
