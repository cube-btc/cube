use crate::constructive::txn::ext::{OutpointExt, TxOutExt};
use bitcoin::{OutPoint, TxOut};

use super::error::LiftV2SBEDecodeError;
use crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2;

/// Decodes the shared `LiftV1` / `LiftV2` payload after the variant byte: keys, outpoint, one `TxOut`.
fn decode_lift_v2_sbe_body(
    payload: &[u8],
) -> Result<([u8; 32], [u8; 32], OutPoint, TxOut), LiftV2SBEDecodeError> {
    let total = payload.len();

    // 1 Decode the 32-byte Schnorr account key.
    if total < 32 {
        return Err(LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForAccountKey { got_total: total });
    }
    let (account_slice, rest) = payload.split_at(32);
    let account_key = account_slice
        .try_into()
        .map_err(|_| LiftV2SBEDecodeError::LiftV2SBEAccountKeyBytesConversionError)?;

    // 2 Decode the 32-byte Schnorr engine key.
    if rest.len() < 32 {
        return Err(LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForEngineKey { got_total: rest.len() });
    }
    let (engine_slice, rest) = rest.split_at(32);
    let engine_key = engine_slice
        .try_into()
        .map_err(|_| LiftV2SBEDecodeError::LiftV2SBEEngineKeyBytesConversionError)?;

    // 3 Decode the 36-byte outpoint (`OutpointExt::from_bytes36`).
    if rest.len() < 36 {
        return Err(LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForOutPoint { got_total: rest.len() });
    }
    let (outpoint_slice, rest) = rest.split_at(36);
    let outpoint_bytes: [u8; 36] = outpoint_slice
        .try_into()
        .map_err(|_| LiftV2SBEDecodeError::LiftV2SBEOutPointBytesConversionError)?;
    let outpoint = OutPoint::from_bytes36(&outpoint_bytes)
        .ok_or(LiftV2SBEDecodeError::LiftV2SBEFailedToDecodeOutPointError)?;

    // 4 Decode the `TxOut` prefix: 8-byte little-endian value.
    if rest.len() < 8 {
        return Err(LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForTxOutValue { got_total: rest.len() });
    }
    let (value_slice, rest) = rest.split_at(8);

    // 5 Read the 1-byte script-pubkey length prefix and the script bytes.
    if rest.is_empty() {
        return Err(
            LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForTxOutScriptLengthPrefix { got_total: 0 },
        );
    }
    let script_len = rest[0] as usize;
    let rest = &rest[1..];
    if rest.len() < script_len {
        return Err(LiftV2SBEDecodeError::LiftV2SBEInsufficientBytesForTxOutScriptPayload {
            got_total: rest.len(),
            script_len,
        });
    }
    let (script_slice, trailing) = rest.split_at(script_len);

    // 6 Rebuild the `TxOut` slice expected by `TxOutExt::from_bytes` and decode.
    let mut txout_bytes = Vec::with_capacity(8 + 1 + script_len);
    txout_bytes.extend_from_slice(value_slice);
    txout_bytes.push(script_len as u8);
    txout_bytes.extend_from_slice(script_slice);
    let txout =
        TxOut::from_bytes(&txout_bytes).ok_or(LiftV2SBEDecodeError::LiftV2SBEFailedToDecodeTxOutError)?;

    // 7 Ensure no bytes trail after the encoded `TxOut`.
    if !trailing.is_empty() {
        return Err(LiftV2SBEDecodeError::LiftV2SBETxOutTrailingBytesInPayload {
            trailing: trailing.len(),
        });
    }

    // 8 Return the decoded fields.
    Ok((account_key, engine_key, outpoint, txout))
}

impl LiftV2 {
    /// Decodes a `LiftV2` from Structural Byte-scope Encoding (SBE) bytes produced by [`LiftV2::encode_sbe`].
    ///
    /// The leading byte must be `0x02` (the `LiftV2` variant discriminant).
    pub fn decode_sbe(bytes: &[u8]) -> Result<LiftV2, LiftV2SBEDecodeError> {
        // 1 Ensure there is at least one byte for the variant discriminant.
        if bytes.is_empty() {
            return Err(LiftV2SBEDecodeError::LiftV2SBEVariantDiscriminantMissingError);
        }

        // 2 Split the discriminant from the payload and verify it is `0x02`.
        let (tag, payload) = bytes.split_at(1);
        if tag[0] != 0x02 {
            return Err(LiftV2SBEDecodeError::LiftV2SBEExpectedVariantDiscriminant0x02Error { got: tag[0] });
        }

        // 3 Decode the payload body (keys, outpoint, `TxOut`).
        let (account_key, engine_key, outpoint, txout) = decode_lift_v2_sbe_body(payload)?;

        // 4 Construct the `LiftV2`.
        Ok(LiftV2::new(account_key, engine_key, outpoint, txout))
    }
}
