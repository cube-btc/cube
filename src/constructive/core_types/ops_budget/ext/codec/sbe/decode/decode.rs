use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;

use super::error::decode_error::OpsBudgetSBEDecodeError;

fn short_val_compact_len(tier: u8) -> Option<usize> {
    match tier {
        0..=3 => Some((tier as usize) + 1),
        _ => None,
    }
}

impl OpsBudget {
    /// Decodes an `OpsBudget` from SBE bytes produced by [`OpsBudget::encode_sbe`].
    pub fn decode_sbe(bytes: &[u8]) -> Result<(OpsBudget, &[u8]), OpsBudgetSBEDecodeError> {
        // 1 Read the presence byte.
        if bytes.is_empty() {
            return Err(OpsBudgetSBEDecodeError::OpsBudgetSBEInsufficientPayloadBytes {
                got: 0,
            });
        }

        match bytes[0] {
            // 2.a Absent — one byte only.
            0 => Ok((OpsBudget::new(None), &bytes[1..])),

            // 2.b Present — tier byte plus compact value bytes.
            1 => {
                if bytes.len() < 2 {
                    return Err(OpsBudgetSBEDecodeError::OpsBudgetSBEInsufficientPayloadBytes {
                        got: bytes.len(),
                    });
                }

                let compact_len = short_val_compact_len(bytes[1]).ok_or(
                    OpsBudgetSBEDecodeError::OpsBudgetSBEInvalidTierByte { got: bytes[1] },
                )?;

                if bytes.len() < 2 + compact_len {
                    return Err(OpsBudgetSBEDecodeError::OpsBudgetSBEInsufficientPayloadBytes {
                        got: bytes.len(),
                    });
                }

                let budget = ShortVal::from_compact_bytes(&bytes[2..2 + compact_len])
                    .ok_or(OpsBudgetSBEDecodeError::OpsBudgetSBEInvalidCompactBytes)?
                    .value();

                Ok((OpsBudget::new(Some(budget)), &bytes[2 + compact_len..]))
            }

            flag => Err(OpsBudgetSBEDecodeError::OpsBudgetSBEInvalidPresenceFlag { got: flag }),
        }
    }
}
