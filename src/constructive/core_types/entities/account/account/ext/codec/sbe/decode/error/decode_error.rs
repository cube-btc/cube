/// Errors that can occur when decoding an `Account` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountSBEDecodeError {
    // SBE variant discriminant.
    /// The buffer was empty, so the leading `Account` SBE variant byte could not be read.
    AccountSBEVariantDiscriminantMissingError,
    /// The leading byte is not a known `Account` SBE variant discriminant (`0x00` for `UnregisteredAccount`, `0x01` for `RegisteredAccount`).
    UnknownAccountSBEVariantDiscriminantByteError(u8),

    // UnregisteredAccount — Schnorr account key (32 bytes).
    /// The payload after `0x00` ended before the 32-byte Schnorr account key to be registered.
    UnregisteredAccountSBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte Schnorr account key from the SBE payload.
    UnregisteredAccountSBEAccountKeyBytesConversionError,
    /// Bytes remained after the 32-byte account key in an `UnregisteredAccount` payload.
    UnregisteredAccountSBETrailingBytesAfterAccountKey { trailing: usize },

    // RegisteredAccount — Schnorr account key (32 bytes) + registery index (8 bytes).
    /// The payload after `0x01` ended before the 32-byte Schnorr account key.
    RegisteredAccountSBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte Schnorr account key from the SBE payload.
    RegisteredAccountSBEAccountKeyBytesConversionError,
    /// The payload after the 32-byte account key ended before the 8-byte little-endian registery index.
    RegisteredAccountSBEInsufficientBytesForRegisteryIndex { got_total: usize },
    /// Failed to assemble the 8-byte registery index from the SBE payload.
    RegisteredAccountSBERegisteryIndexBytesConversionError,
    /// Bytes remained after the account key and registery index in a `RegisteredAccount` payload.
    RegisteredAccountSBETrailingBytesAfterRegisteryIndex { trailing: usize },
}
