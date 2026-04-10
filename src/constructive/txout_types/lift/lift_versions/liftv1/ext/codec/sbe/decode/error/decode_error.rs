/// Errors that can occur when decoding a `LiftV1` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftV1SBEDecodeError {
    // SBE variant discriminant (leading byte).
    /// The buffer was empty, so the leading `LiftV1` SBE variant byte could not be read.
    LiftV1SBEVariantDiscriminantMissingError,
    /// Decoding as `LiftV1` requires a leading `0x01` byte before the payload.
    LiftV1SBEExpectedVariantDiscriminant0x01Error { got: u8 },

    // Shared payload: Schnorr account key (32 bytes).
    /// The payload ended before the 32-byte account key.
    LiftV1SBEInsufficientBytesForAccountKey { got_total: usize },
    /// Failed to assemble the 32-byte account key from the SBE payload.
    LiftV1SBEAccountKeyBytesConversionError,

    // Shared payload: Schnorr engine key (32 bytes).
    /// The payload ended before the 32-byte engine key.
    LiftV1SBEInsufficientBytesForEngineKey { got_total: usize },
    /// Failed to assemble the 32-byte engine key from the SBE payload.
    LiftV1SBEEngineKeyBytesConversionError,

    // Shared payload: `OutPoint` (36 bytes via `OutpointExt::bytes_36`).
    /// The payload ended before the 36-byte outpoint encoding.
    LiftV1SBEInsufficientBytesForOutPoint { got_total: usize },
    /// Failed to assemble the 36-byte outpoint slice from the SBE payload.
    LiftV1SBEOutPointBytesConversionError,
    /// `OutPoint::from_bytes36` rejected the 36-byte outpoint encoding.
    LiftV1SBEFailedToDecodeOutPointError,

    // Shared payload: `TxOut` tail (8-byte value LE, 1-byte script length, script bytes).
    /// The payload ended before the 8-byte little-endian satoshi value of the `TxOut`.
    LiftV1SBEInsufficientBytesForTxOutValue { got_total: usize },
    /// The payload ended before the 1-byte script-pubkey length prefix after the `TxOut` value.
    LiftV1SBEInsufficientBytesForTxOutScriptLengthPrefix { got_total: usize },
    /// The script-pubkey length prefix plus fixed `TxOut` prefix exceeds the remaining payload.
    LiftV1SBEInsufficientBytesForTxOutScriptPayload { got_total: usize, script_len: usize },
    /// `TxOut::from_bytes` failed for the encoded `TxOut` slice.
    LiftV1SBEFailedToDecodeTxOutError,
    /// Bytes remained after the fully encoded `TxOut` within the lift payload.
    LiftV1SBETxOutTrailingBytesInPayload { trailing: usize },
}
