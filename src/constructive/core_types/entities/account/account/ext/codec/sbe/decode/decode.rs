use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::account::registered_account::registered_account::RegisteredAccount;
use crate::constructive::entity::account::account::unregistered_account::unregistered_account::UnregisteredAccount;

use super::error::AccountSBEDecodeError;

impl Account {
    /// Decodes an `Account` from Structural Byte-scope Encoding (SBE) bytes produced by [`Account::encode_sbe`].
    pub fn decode_sbe(bytes: &[u8]) -> Result<Account, AccountSBEDecodeError> {
        // 1 Ensure there is at least one byte for the SBE variant discriminant.
        if bytes.is_empty() {
            return Err(AccountSBEDecodeError::AccountSBEVariantDiscriminantMissingError);
        }

        // 2 Read the SBE variant discriminant.
        let variant_tag = bytes[0];

        // 3 Slice the payload after the discriminant.
        let payload = &bytes[1..];

        // 4 Match on the SBE variant discriminant.
        match variant_tag {
            // 4.a The `Account` is an `UnregisteredAccount` (`0x00`).
            0x00 => {
                // 4.a.1 Ensure the payload holds the 32-byte Schnorr account key.
                if payload.len() < 32 {
                    return Err(
                        AccountSBEDecodeError::UnregisteredAccountSBEInsufficientBytesForAccountKey {
                            got_total: payload.len(),
                        },
                    );
                }

                // 4.a.2 Decode the account key to be registered.
                let account_key_to_be_registered: [u8; 32] = payload[0..32]
                    .try_into()
                    .map_err(|_| AccountSBEDecodeError::UnregisteredAccountSBEAccountKeyBytesConversionError)?;

                // 4.a.3 Ensure no trailing bytes after the account key.
                if payload.len() != 32 {
                    return Err(
                        AccountSBEDecodeError::UnregisteredAccountSBETrailingBytesAfterAccountKey {
                            trailing: payload.len() - 32,
                        },
                    );
                }

                // 4.a.4 Construct and return the unregistered `Account`.
                Ok(Account::UnregisteredAccount(UnregisteredAccount::new(
                    account_key_to_be_registered,
                )))
            }

            // 4.b The `Account` is a `RegisteredAccount` (`0x01`).
            0x01 => {
                // 4.b.1 Ensure the payload holds the 32-byte Schnorr account key.
                if payload.len() < 32 {
                    return Err(
                        AccountSBEDecodeError::RegisteredAccountSBEInsufficientBytesForAccountKey {
                            got_total: payload.len(),
                        },
                    );
                }

                // 4.b.2 Decode the account key.
                let account_key: [u8; 32] = payload[0..32]
                    .try_into()
                    .map_err(|_| AccountSBEDecodeError::RegisteredAccountSBEAccountKeyBytesConversionError)?;

                // 4.b.3 Ensure the payload holds the 8-byte little-endian registery index after the key.
                if payload.len() < 32 + 8 {
                    return Err(
                        AccountSBEDecodeError::RegisteredAccountSBEInsufficientBytesForRegisteryIndex {
                            got_total: payload.len(),
                        },
                    );
                }

                // 4.b.4 Decode the registery index.
                let registery_index = u64::from_le_bytes(
                    payload[32..40]
                        .try_into()
                        .map_err(|_| AccountSBEDecodeError::RegisteredAccountSBERegisteryIndexBytesConversionError)?,
                );

                // 4.b.5 Ensure no trailing bytes after the registery index.
                if payload.len() != 32 + 8 {
                    return Err(
                        AccountSBEDecodeError::RegisteredAccountSBETrailingBytesAfterRegisteryIndex {
                            trailing: payload.len() - (32 + 8),
                        },
                    );
                }

                // 4.b.6 Construct and return the registered `Account`.
                Ok(Account::RegisteredAccount(RegisteredAccount::new(
                    account_key,
                    registery_index,
                )))
            }

            // 4.c The discriminant does not match any known variant.
            b => Err(AccountSBEDecodeError::UnknownAccountSBEVariantDiscriminantByteError(b)),
        }
    }
}
