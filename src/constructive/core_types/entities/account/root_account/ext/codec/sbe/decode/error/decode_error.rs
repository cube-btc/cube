/// Enum to represent errors that can occur when decoding a `RootAccount` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootAccountSBEDecodeError {
    // SBE variant discriminant.
    /// The buffer was empty, so the leading `RootAccount` SBE variant byte could not be read.
    RootAccountSBEVariantDiscriminantMissingError,
    /// The leading byte is not a known `RootAccount` SBE variant discriminant (`0x00`, `0x01`, or `0x02`).
    UnknownRootAccountSBEVariantDiscriminantByteError(u8),

    // UnregisteredRootAccount — Schnorr account key (32 bytes).
    /// The payload after `0x00` ended before the 32-byte Schnorr account key to be registered.
    UnregisteredRootAccountSBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte Schnorr account key to be registered from the SBE payload.
    UnregisteredRootAccountSBEAccountKeyBytesConversionError,

    // UnregisteredRootAccount — BLS public key (48 bytes).
    /// The payload after `0x00` ended before the 48-byte BLS key to be configured.
    UnregisteredRootAccountSBEInsufficientBytesForBlsKey { got_total: usize },
    /// Failed to assemble the 48-byte BLS key to be configured from the SBE payload.
    UnregisteredRootAccountSBEBlsKeyBytesConversionError,

    // UnregisteredRootAccount — flame presence prefix and trailing section.
    /// The payload ended before the flame-presence prefix byte after account and BLS keys.
    UnregisteredRootAccountSBEInsufficientBytesForFlamePresencePrefix { got_total: usize },
    /// The flame-presence prefix was not `0x00` (absent) or `0x01` (present).
    UnregisteredRootAccountSBEFlamePresencePrefixInvalidError(u8),
    /// After `0x00` (no flame config), the trailing section length was not exactly 65 bytes (64-byte signature only).
    UnregisteredRootAccountSBEFlameAbsentTrailingSectionLengthError { got_tail: usize },
    /// Failed to assemble the 64-byte BLS authorization Schnorr signature when the flame config is absent.
    UnregisteredRootAccountSBEFlameAbsentAuthorizationSignatureBytesConversionError,
    /// With `0x01` (flame present), the tail was shorter than one flame byte plus the 64-byte signature.
    UnregisteredRootAccountSBEInsufficientBytesWhenFlameConfigPresent { got_total: usize },
    /// `FMAccountFlameConfig::from_bytes` failed for the flame payload embedded in the unregistered `RootAccount` SBE.
    UnregisteredRootAccountSBEFailedToDecodeFlameConfigError,
    /// Failed to assemble the 64-byte BLS authorization Schnorr signature when the flame config is present.
    UnregisteredRootAccountSBEFlamePresentAuthorizationSignatureBytesConversionError,

    // RegisteredButUnconfiguredRootAccount — Schnorr account key (32 bytes).
    /// The payload after `0x01` ended before the 32-byte Schnorr account key.
    RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte Schnorr account key from the SBE payload.
    RegisteredButUnconfiguredRootAccountSBEAccountKeyBytesConversionError,

    // RegisteredButUnconfiguredRootAccount — registery index (8 bytes).
    /// The payload after `0x01` ended before the 8-byte little-endian registery index.
    RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForRegisteryIndex { got_total: usize },
    /// Failed to assemble the 8-byte registery index from the SBE payload.
    RegisteredButUnconfiguredRootAccountSBERegisteryIndexBytesConversionError,

    // RegisteredButUnconfiguredRootAccount — BLS public key (48 bytes).
    /// The payload after `0x01` ended before the 48-byte BLS key to be configured.
    RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForBlsKey { got_total: usize },
    /// Failed to assemble the 48-byte BLS key to be configured from the SBE payload.
    RegisteredButUnconfiguredRootAccountSBEBlsKeyBytesConversionError,

    // RegisteredButUnconfiguredRootAccount — flame presence prefix and trailing section.
    /// The payload ended before the flame-presence prefix byte after account key, index, and BLS key.
    RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForFlamePresencePrefix {
        got_total: usize,
    },
    /// The flame-presence prefix was not `0x00` (absent) or `0x01` (present).
    RegisteredButUnconfiguredRootAccountSBEFlamePresencePrefixInvalidError(u8),
    /// After `0x00` (no flame config), the trailing section length was not exactly 65 bytes (64-byte signature only).
    RegisteredButUnconfiguredRootAccountSBEFlameAbsentTrailingSectionLengthError { got_tail: usize },
    /// Failed to assemble the 64-byte BLS authorization Schnorr signature when the flame config is absent.
    RegisteredButUnconfiguredRootAccountSBEFlameAbsentAuthorizationSignatureBytesConversionError,
    /// With `0x01` (flame present), the tail was shorter than one flame byte plus the 64-byte signature.
    RegisteredButUnconfiguredRootAccountSBEInsufficientBytesWhenFlameConfigPresent {
        got_total: usize,
    },
    /// `FMAccountFlameConfig::from_bytes` failed for the flame payload embedded in the registered-but-unconfigured `RootAccount` SBE.
    RegisteredButUnconfiguredRootAccountSBEFailedToDecodeFlameConfigError,
    /// Failed to assemble the 64-byte BLS authorization Schnorr signature when the flame config is present.
    RegisteredButUnconfiguredRootAccountSBEFlamePresentAuthorizationSignatureBytesConversionError,

    // RegisteredAndConfiguredRootAccount — fixed payload after `0x02`.
    /// The payload after `0x02` was not exactly 88 bytes (`32` account + `8` registery index + `48` BLS key).
    RegisteredAndConfiguredRootAccountSBEPayloadLengthError {
        got_payload: usize,
        expected_payload: usize,
    },
    /// Failed to assemble the 32-byte Schnorr account key from the registered-and-configured SBE payload.
    RegisteredAndConfiguredRootAccountSBEAccountKeyBytesConversionError,
    /// Failed to assemble the 8-byte little-endian registery index from the registered-and-configured SBE payload.
    RegisteredAndConfiguredRootAccountSBERegisteryIndexBytesConversionError,
    /// Failed to assemble the 48-byte BLS key from the registered-and-configured SBE payload.
    RegisteredAndConfiguredRootAccountSBEBlsKeyBytesConversionError,
}
