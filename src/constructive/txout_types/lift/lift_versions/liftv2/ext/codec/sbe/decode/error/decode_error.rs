/// Errors that can occur when decoding a `LiftV2` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftV2SBEDecodeError {
    // SBE variant discriminant (leading byte).
    /// The buffer was empty, so the leading `LiftV2` SBE variant byte could not be read.
    LiftV2SBEVariantDiscriminantMissingError,
    /// Decoding as `LiftV2` requires a leading `0x01` byte before the payload.
    LiftV2SBEExpectedVariantDiscriminant0x01Error { got: u8 },

    // Shared payload: Schnorr account key (32 bytes).
    /// The payload ended before the 32-byte account key.
    LiftV2SBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte account key from the SBE payload.
    LiftV2SBEAccountKeyBytesConversionError,

    // Shared payload: Schnorr engine key (32 bytes).
    /// The payload ended before the 32-byte engine key.
    LiftV2SBEInsufficientBytesForEngineKey { got_total: usize },
    /// Failed to assemble the 32-byte engine key from the SBE payload.
    LiftV2SBEEngineKeyBytesConversionError,

    // Shared payload: `OutPoint` (36 bytes via `OutpointExt::bytes_36`).
    /// The payload ended before the 36-byte outpoint encoding.
    LiftV2SBEInsufficientBytesForOutPoint { got_total: usize },
    /// Failed to assemble the 36-byte outpoint slice from the SBE payload.
    LiftV2SBEOutPointBytesConversionError,
    /// `OutPoint::from_bytes36` rejected the 36-byte outpoint encoding.
    LiftV2SBEFailedToDecodeOutPointError,

    // Shared payload: `TxOut` tail (8-byte value LE, 1-byte script length, script bytes).
    /// The payload ended before the 8-byte little-endian satoshi value of the `TxOut`.
    LiftV2SBEInsufficientBytesForTxOutValue { got_total: usize },
    /// The payload ended before the 1-byte script-pubkey length prefix after the `TxOut` value.
    LiftV2SBEInsufficientBytesForTxOutScriptLengthPrefix { got_total: usize },
    /// The script-pubkey length prefix plus fixed `TxOut` prefix exceeds the remaining payload.
    LiftV2SBEInsufficientBytesForTxOutScriptPayload { got_total: usize, script_len: usize },
    /// `TxOut::from_bytes` failed for the encoded `TxOut` slice.
    LiftV2SBEFailedToDecodeTxOutError,
    /// Bytes remained after the fully encoded `TxOut` within the lift payload.
    LiftV2SBETxOutTrailingBytesInPayload { trailing: usize },
}
