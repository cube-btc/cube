use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;

use super::error::decode_error::OpsBudgetSBEDecodeError;

impl OpsBudget {
    /// Decodes an `OpsBudget` from Structural Byte-scope Encoding (SBE) bytes produced by [`OpsBudget::encode_sbe`].
    ///
    /// The buffer must be exactly five bytes (presence flag + little-endian `u32` budget).
    pub fn decode_sbe(bytes: &[u8]) -> Result<OpsBudget, OpsBudgetSBEDecodeError> {
        let arr: [u8; 5] =
            bytes
                .try_into()
                .map_err(|_| OpsBudgetSBEDecodeError::OpsBudgetSBEInvalidPayloadLength {
                    got: bytes.len(),
                })?;

        match arr[0] {
            0 => Ok(OpsBudget::new(None)),
            1 => {
                let budget = u32::from_le_bytes([arr[1], arr[2], arr[3], arr[4]]);
                Ok(OpsBudget::new(Some(budget)))
            }
            flag => Err(OpsBudgetSBEDecodeError::OpsBudgetSBEInvalidPresenceFlag {
                got: flag,
            }),
        }
    }
}
