use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;

use super::error::decode_error::MoveSBEDecodeError;

impl Move {
    /// Decodes a `Move` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Move, MoveSBEDecodeError> {
        // 1 Validate the leading `Move` entry kind tag.
        if bytes.is_empty() {
            return Err(MoveSBEDecodeError::MoveSBEInsufficientBytesForFromLengthPrefix {
                got_total: 0,
            });
        }
        if bytes[0] != 0x00 {
            return Err(MoveSBEDecodeError::InvalidEntryKindByteError {
                expected: 0x00,
                got: bytes[0],
            });
        }

        // 2 Decode the sender (`from`) payload length prefix.
        if bytes.len() < 5 {
            return Err(MoveSBEDecodeError::MoveSBEInsufficientBytesForFromLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let from_len = u32::from_le_bytes(bytes[1..5].try_into().map_err(|_| {
            MoveSBEDecodeError::MoveSBEFromLengthPrefixBytesConversionError
        })?) as usize;
        let after_from_len_prefix = &bytes[5..];
        if after_from_len_prefix.len() < from_len {
            return Err(MoveSBEDecodeError::MoveSBEFromLengthPrefixExceedsPayload {
                from_len,
                got_after_prefix: after_from_len_prefix.len(),
            });
        }

        // 3 Decode sender `RootAccount`.
        let (from_slice, after_from) = after_from_len_prefix.split_at(from_len);
        let from =
            RootAccount::decode_sbe(from_slice).map_err(MoveSBEDecodeError::MoveSBEFromRootAccount)?;

        // 4 Decode the receiver (`to`) payload length prefix.
        if after_from.len() < 4 {
            return Err(MoveSBEDecodeError::MoveSBEInsufficientBytesForToLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let to_len = u32::from_le_bytes(after_from[0..4].try_into().map_err(|_| {
            MoveSBEDecodeError::MoveSBEToLengthPrefixBytesConversionError
        })?) as usize;
        let after_to_len_prefix = &after_from[4..];
        if after_to_len_prefix.len() < to_len {
            return Err(MoveSBEDecodeError::MoveSBEToLengthPrefixExceedsPayload {
                to_len,
                got_after_prefix: after_to_len_prefix.len(),
            });
        }

        // 5 Decode receiver `Account`.
        let (to_slice, after_to) = after_to_len_prefix.split_at(to_len);
        let to = Account::decode_sbe(to_slice).map_err(MoveSBEDecodeError::MoveSBEToAccount)?;

        // 6 Decode amount.
        if after_to.len() < 4 {
            return Err(MoveSBEDecodeError::MoveSBEInsufficientBytesForAmount {
                got_total: bytes.len(),
            });
        }
        let amount = u32::from_le_bytes(after_to[0..4].try_into().map_err(|_| {
            MoveSBEDecodeError::MoveSBEAmountBytesConversionError
        })?);

        // 7 Decode target.
        if after_to.len() < 12 {
            return Err(MoveSBEDecodeError::MoveSBEInsufficientBytesForTarget {
                got_total: bytes.len(),
            });
        }
        let target = Target::decode_sbe(&after_to[4..12]).map_err(MoveSBEDecodeError::MoveSBETarget)?;

        // 8 Reject trailing bytes.
        let tail = &after_to[12..];
        if !tail.is_empty() {
            return Err(MoveSBEDecodeError::MoveSBETrailingBytesAfterMove {
                trailing: tail.len(),
            });
        }

        // 9 Return decoded `Move`.
        Ok(Move::new(from, to, amount, target))
    }
}
