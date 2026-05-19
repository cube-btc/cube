use crate::constructive::calldata::element::calldata_element::CalldataElement;
use crate::constructive::calldata::element::sbe::decode::error::decode_errors::{
    BoolSBEDecodeError, BytesSBEDecodeError, CalldataElementSBEDecodeError,
    CallAccountSBEDecodeError, CallContractSBEDecodeError, PayableSBEDecodeError, U16SBEDecodeError,
    U32SBEDecodeError, U64SBEDecodeError, U8SBEDecodeError, VarbytesSBEDecodeError,
};
use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entity::contract::ext::codec::sbe::encode::encode::CONTRACT_SBE_LEN;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;

const MAX_VARBYTES_LEN: u16 = 4095;

fn short_val_compact_len(tier: u8) -> Option<usize> {
    match tier {
        0..=3 => Some((tier as usize) + 1),
        _ => None,
    }
}

fn long_val_compact_len(tier: u8) -> Option<usize> {
    match tier {
        0..=7 => Some((tier as usize) + 1),
        _ => None,
    }
}

fn decode_short_val_sbe(
    payload: &[u8],
) -> Result<(ShortVal, usize), CalldataElementSBEDecodeError> {
    if payload.is_empty() {
        return Err(CalldataElementSBEDecodeError::U32(U32SBEDecodeError::InsufficientPayloadBytes {
            got: 0,
        }));
    }

    let compact_len = short_val_compact_len(payload[0]).ok_or(
        CalldataElementSBEDecodeError::U32(U32SBEDecodeError::InvalidTierByte(payload[0])),
    )?;

    if payload.len() < 1 + compact_len {
        return Err(CalldataElementSBEDecodeError::U32(U32SBEDecodeError::InsufficientPayloadBytes {
            got: payload.len(),
        }));
    }

    let short_val = ShortVal::from_compact_bytes(&payload[1..1 + compact_len]).ok_or(
        CalldataElementSBEDecodeError::U32(U32SBEDecodeError::InvalidCompactBytes),
    )?;

    Ok((short_val, 1 + compact_len))
}

fn decode_long_val_sbe(payload: &[u8]) -> Result<(LongVal, usize), CalldataElementSBEDecodeError> {
    if payload.is_empty() {
        return Err(CalldataElementSBEDecodeError::U64(U64SBEDecodeError::InsufficientPayloadBytes {
            got: 0,
        }));
    }

    let compact_len = long_val_compact_len(payload[0]).ok_or(
        CalldataElementSBEDecodeError::U64(U64SBEDecodeError::InvalidTierByte(payload[0])),
    )?;

    if payload.len() < 1 + compact_len {
        return Err(CalldataElementSBEDecodeError::U64(U64SBEDecodeError::InsufficientPayloadBytes {
            got: payload.len(),
        }));
    }

    let long_val = LongVal::from_compact_bytes(&payload[1..1 + compact_len]).ok_or(
        CalldataElementSBEDecodeError::U64(U64SBEDecodeError::InvalidCompactBytes),
    )?;

    Ok((long_val, 1 + compact_len))
}

fn parse_element_type(bytes: &[u8]) -> Result<(CalldataElementType, usize), CalldataElementSBEDecodeError> {
    if bytes.is_empty() {
        return Err(CalldataElementSBEDecodeError::CalldataElementSBEEmptyBufferError);
    }

    match bytes[0] {
        0x00 => Ok((CalldataElementType::U8, 1)),
        0x01 => Ok((CalldataElementType::U16, 1)),
        0x02 => Ok((CalldataElementType::U32, 1)),
        0x03 => Ok((CalldataElementType::U64, 1)),
        0x04 => Ok((CalldataElementType::Bool, 1)),
        0x05 => Ok((CalldataElementType::Account, 1)),
        0x06 => Ok((CalldataElementType::Contract, 1)),
        0x07 => {
            if bytes.len() < 2 {
                return Err(CalldataElementSBEDecodeError::CalldataElementSBEBytesTypeMissingLengthIndexError);
            }
            Ok((CalldataElementType::Bytes(bytes[1]), 2))
        }
        0x08 => Ok((CalldataElementType::Varbytes, 1)),
        0x09 => Ok((CalldataElementType::Payable, 1)),
        b => Err(CalldataElementSBEDecodeError::CalldataElementSBEUnknownTypeTagError(b)),
    }
}

fn account_sbe_payload_len(account: &Account) -> usize {
    match account {
        Account::UnregisteredAccount(_) => 1 + 32,
        Account::RegisteredAccount(_) => 1 + 32 + 8,
    }
}

impl CalldataElement {
    /// Decodes a `CalldataElement` from SBE bytes produced by [`CalldataElement::encode_sbe`].
    ///
    /// Returns the decoded element and the unconsumed suffix of `bytes`.
    pub fn decode_sbe(bytes: &[u8]) -> Result<(Self, &[u8]), CalldataElementSBEDecodeError> {
        let (element_type, header_len) = parse_element_type(bytes)?;
        let payload = &bytes[header_len..];

        let (element, payload_consumed) = match element_type {
            CalldataElementType::U8 => {
                if payload.is_empty() {
                    return Err(CalldataElementSBEDecodeError::U8(U8SBEDecodeError::InsufficientPayloadBytes {
                        got: 0,
                    }));
                }
                (CalldataElement::U8(payload[0]), 1)
            }
            CalldataElementType::U16 => {
                if payload.len() < 2 {
                    return Err(CalldataElementSBEDecodeError::U16(U16SBEDecodeError::InsufficientPayloadBytes {
                        got: payload.len(),
                    }));
                }
                let value = u16::from_le_bytes(
                    payload[0..2].try_into().map_err(|_| {
                        CalldataElementSBEDecodeError::U16(U16SBEDecodeError::BytesConversionError)
                    })?,
                );
                (CalldataElement::U16(value), 2)
            }
            CalldataElementType::U32 => {
                let (short_val, consumed) = decode_short_val_sbe(payload)?;
                (CalldataElement::U32(short_val), consumed)
            }
            CalldataElementType::U64 => {
                let (long_val, consumed) = decode_long_val_sbe(payload)?;
                (CalldataElement::U64(long_val), consumed)
            }
            CalldataElementType::Bool => {
                if payload.is_empty() {
                    return Err(CalldataElementSBEDecodeError::Bool(BoolSBEDecodeError::InsufficientPayloadBytes {
                        got: 0,
                    }));
                }
                let value = match payload[0] {
                    0x00 => false,
                    0x01 => true,
                    b => {
                        return Err(CalldataElementSBEDecodeError::Bool(
                            BoolSBEDecodeError::InvalidBoolByte(b),
                        ));
                    }
                };
                (CalldataElement::Bool(value), 1)
            }
            CalldataElementType::Account => {
                let account = Account::decode_sbe(payload).map_err(|e| {
                    CalldataElementSBEDecodeError::Account(CallAccountSBEDecodeError::AccountSBEDecodeError(e))
                })?;
                let consumed = account_sbe_payload_len(&account);
                (CalldataElement::Account(account), consumed)
            }
            CalldataElementType::Contract => {
                let contract = Contract::decode_sbe(payload).map_err(|e| {
                    CalldataElementSBEDecodeError::Contract(CallContractSBEDecodeError::ContractSBEDecodeError(e))
                })?;
                (CalldataElement::Contract(contract), CONTRACT_SBE_LEN)
            }
            CalldataElementType::Bytes(index) => {
                let byte_length = index as usize + 1;
                if byte_length < 1 || byte_length > 256 {
                    return Err(CalldataElementSBEDecodeError::Bytes(BytesSBEDecodeError::InvalidBytesLength(
                        byte_length,
                    )));
                }
                if payload.len() < byte_length {
                    return Err(CalldataElementSBEDecodeError::Bytes(
                        BytesSBEDecodeError::InsufficientPayloadBytes {
                            expected: byte_length,
                            got: payload.len(),
                        },
                    ));
                }
                (
                    CalldataElement::Bytes(payload[0..byte_length].to_vec()),
                    byte_length,
                )
            }
            CalldataElementType::Varbytes => {
                if payload.len() < 2 {
                    return Err(CalldataElementSBEDecodeError::Varbytes(
                        VarbytesSBEDecodeError::InsufficientPayloadBytesForLength { got: payload.len() },
                    ));
                }
                let byte_length = u16::from_le_bytes([payload[0], payload[1]]);
                if byte_length > MAX_VARBYTES_LEN {
                    return Err(CalldataElementSBEDecodeError::Varbytes(
                        VarbytesSBEDecodeError::ByteLengthGreaterThan4095Error(byte_length),
                    ));
                }
                let byte_length = byte_length as usize;
                if payload.len() < 2 + byte_length {
                    return Err(CalldataElementSBEDecodeError::Varbytes(
                        VarbytesSBEDecodeError::InsufficientPayloadBytesForData {
                            expected: byte_length,
                            got: payload.len().saturating_sub(2),
                        },
                    ));
                }
                (
                    CalldataElement::Varbytes(payload[2..2 + byte_length].to_vec()),
                    2 + byte_length,
                )
            }
            CalldataElementType::Payable => {
                let (short_val, consumed) = decode_short_val_sbe(payload).map_err(|e| match e {
                    CalldataElementSBEDecodeError::U32(inner) => {
                        CalldataElementSBEDecodeError::Payable(match inner {
                            U32SBEDecodeError::InsufficientPayloadBytes { got } => {
                                PayableSBEDecodeError::InsufficientPayloadBytes { got }
                            }
                            U32SBEDecodeError::InvalidTierByte(b) => PayableSBEDecodeError::InvalidTierByte(b),
                            U32SBEDecodeError::InvalidCompactBytes => PayableSBEDecodeError::InvalidCompactBytes,
                        })
                    }
                    other => other,
                })?;
                (CalldataElement::Payable(short_val), consumed)
            }
        };

        let total_consumed = header_len + payload_consumed;
        Ok((element, &bytes[total_consumed..]))
    }
}
