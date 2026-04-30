use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::swapout::ext::codec::sbe::decode::error::decode_error::SwapoutSBEDecodeError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;

impl Swapout {
    /// Decodes a `Swapout` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Self, SwapoutSBEDecodeError> {
        if bytes.is_empty() {
            return Err(SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: 0,
            });
        }
        if bytes[0] != 0x05 {
            return Err(SwapoutSBEDecodeError::InvalidEntryKindByteError {
                expected: 0x05,
                got: bytes[0],
            });
        }
        if bytes.len() < 5 {
            return Err(SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: bytes.len(),
            });
        }

        let root_len_u32 = u32::from_le_bytes(bytes[1..5].try_into().map_err(|_| {
            SwapoutSBEDecodeError::SwapoutSBERootAccountLengthPrefixBytesConversionError
        })?);
        let root_len = root_len_u32 as usize;
        let after_root_prefix = &bytes[5..];
        if after_root_prefix.len() < root_len {
            return Err(SwapoutSBEDecodeError::SwapoutSBERootAccountLengthPrefixExceedsPayload {
                root_len,
                got_after_prefix: after_root_prefix.len(),
            });
        }

        let (root_slice, after_root) = after_root_prefix.split_at(root_len);
        let root_account = RootAccount::decode_sbe(root_slice)
            .map_err(|_| SwapoutSBEDecodeError::SwapoutSBERootAccountDecodeError)?;

        if after_root.len() < 12 {
            return Err(SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForTargetAndAmount {
                got_total: bytes.len(),
            });
        }

        let target = Target::decode_sbe(&after_root[..8]).map_err(|_| {
            SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForTargetAndAmount {
                got_total: bytes.len(),
            }
        })?;
        let amount = u32::from_le_bytes(after_root[8..12].try_into().map_err(|_| {
            SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForTargetAndAmount {
                got_total: bytes.len(),
            }
        })?);

        let pinless_slice = &after_root[12..];
        if pinless_slice.is_empty() {
            return Err(SwapoutSBEDecodeError::SwapoutSBEInsufficientBytesForPinlessSelfKind {
                got_total: bytes.len(),
            });
        }

        let pinless_kind = pinless_slice[0];
        let pinless_self = match pinless_kind {
            // Default: no further payload.
            0x00 => {
                if pinless_slice.len() > 1 {
                    return Err(SwapoutSBEDecodeError::SwapoutSBETrailingBytesAfterSwapout {
                        trailing: pinless_slice.len() - 1,
                    });
                }
                PinlessSelf::new_default(root_account.account_key(), None)
            }
            // Unknown: remaining bytes are custom scriptpubkey.
            0x01 => {
                let custom_scriptpubkey = pinless_slice[1..].to_vec();
                PinlessSelf::new_unknown(custom_scriptpubkey, None)
            }
            b => return Err(SwapoutSBEDecodeError::SwapoutSBEInvalidPinlessSelfKindByte(b)),
        };

        Ok(Swapout::new(root_account, amount, target, pinless_self))
    }
}
