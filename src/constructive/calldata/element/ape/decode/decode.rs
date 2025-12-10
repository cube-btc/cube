use crate::constructive::calldata::element::ape::decode::error::decode_errors::{
    BoolAPEDecodeError, BytesAPEDecodeError, CallAccountAPEDecodeError, CallContractAPEDecodeError,
    CalldataElementAPEDecodeError, PayableAPEDecodeError, U16APEDecodeError, U32APEDecodeError,
    U64APEDecodeError, U8APEDecodeError, VarbytesAPEDecodeError,
};
use crate::constructive::calldata::element::element::CalldataElement;
use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::maybe_common::maybe_common::maybe_common::MaybeCommon;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;

use bit_vec::BitVec;

impl CalldataElement {
    /// Decodes a `CallElement` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// This function decodes a `CallElement` from an Airly Payload Encoding (APE) bit stream.
    /// The `CallElement` can be a u8, u16, u32, u64, bool, account, contract, bytes, varbytes, or payable.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `element_type` - The type of the `CallElement`.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_rank_as_longval` - Whether to decode the rank value as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'_>,
        element_type: CalldataElementType,
        registery_manager: &REGISTERY_MANAGER,
        decode_rank_as_longval: bool,
    ) -> Result<Self, CalldataElementAPEDecodeError> {
        // Match on the calldata element type.
        match element_type {
            // Decode the u8.
            CalldataElementType::U8 => {
                // Create a new bit vector.
                let mut bits = BitVec::new();

                // Collect 8 bits.
                for _ in 0..8 {
                    bits.push(bit_stream.next().ok_or(CalldataElementAPEDecodeError::U8(
                        U8APEDecodeError::Collect8BitsError,
                    ))?);
                }

                // Convert to byte.
                let byte: [u8; 1] = bits.to_bytes().try_into().map_err(|_| {
                    CalldataElementAPEDecodeError::U8(U8APEDecodeError::ConvertToByteError)
                })?;

                // Convert byte to a u8.
                let value = byte[0];

                // Construct the `CalldataElement`.
                let element = CalldataElement::U8(value);

                // Return the element.
                Ok(element)
            }

            // Decode the u16.
            CalldataElementType::U16 => {
                // Create a new bit vector.
                let mut bits = BitVec::new();

                // Collect 16 bits.
                for _ in 0..16 {
                    bits.push(bit_stream.next().ok_or(CalldataElementAPEDecodeError::U16(
                        U16APEDecodeError::Collect16BitsError,
                    ))?);
                }

                // Convert to bytes.
                let bytes: [u8; 2] = bits.to_bytes().try_into().map_err(|_| {
                    CalldataElementAPEDecodeError::U16(U16APEDecodeError::ConvertToBytesError)
                })?;

                // Convert the bytes to a u16.
                let value = u16::from_le_bytes(bytes);

                // Construct the `CalldataElement`.
                let element = CalldataElement::U16(value);

                // Return the element.
                Ok(element)
            }

            // Decode the u32.
            CalldataElementType::U32 => {
                // Decode the `ShortVal` from `MaybeCommon<ShortVal>`.
                let short_val = MaybeCommon::<ShortVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CalldataElementAPEDecodeError::U32(
                            U32APEDecodeError::MaybeCommonShortValAPEDecodingError(e),
                        )
                    })?
                    .value();

                // Construct the `CalldataElement`.
                let element = CalldataElement::U32(short_val);

                // Return the element.
                Ok(element)
            }

            // Decode the u64.
            CalldataElementType::U64 => {
                // Decode the `LongVal` from `MaybeCommon<LongVal>`.
                let long_val = MaybeCommon::<LongVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CalldataElementAPEDecodeError::U64(
                            U64APEDecodeError::MaybeCommonLongValAPEDecodingError(e),
                        )
                    })?
                    .value();

                let element = CalldataElement::U64(long_val);

                // Return the element.
                Ok(element)
            }

            // Decode the bool.
            CalldataElementType::Bool => {
                // Collect the bool value by iterating over a single bit.
                let bool = bit_stream
                    .next()
                    .ok_or(CalldataElementAPEDecodeError::Bool(
                        BoolAPEDecodeError::CollectBoolBitError,
                    ))?;

                // Construct the `CalldataElement`.
                let element = CalldataElement::Bool(bool);

                // Return the element.
                Ok(element)
            }

            // Decode the `Account`.
            CalldataElementType::Account => {
                // Decode the `Account`.
                let account =
                    Account::decode_ape(bit_stream, &registery_manager, decode_rank_as_longval)
                        .await
                        .map_err(|e| {
                            CalldataElementAPEDecodeError::Account(
                                CallAccountAPEDecodeError::AccountAPEDecodeError(e),
                            )
                        })?;

                // Construct the `CalldataElement`.
                let element = CalldataElement::Account(account);

                // Return the element.
                Ok(element)
            }

            // Decode the `Contract`.
            CalldataElementType::Contract => {
                // Decode the `Contract`.
                let contract =
                    Contract::decode_ape(bit_stream, &registery_manager, decode_rank_as_longval)
                        .await
                        .map_err(|e| {
                            CalldataElementAPEDecodeError::Contract(
                                CallContractAPEDecodeError::ContractAPEDecodeError(e),
                            )
                        })?;

                // Construct the `CallElement`.
                let element = CalldataElement::Contract(contract);

                // Return the element.
                Ok(element)
            }

            // Decode the `Bytes1-256`.
            CalldataElementType::Bytes(index) => {
                // Byte length is the index + 1.
                let byte_length = index as usize + 1;

                // Check if the data length is valid.
                if byte_length < 1 || byte_length > 256 {
                    return Err(CalldataElementAPEDecodeError::Bytes(
                        BytesAPEDecodeError::InvalidBytesLength(byte_length),
                    ));
                }

                // Get the number of bits to collect.
                let bit_length = byte_length as usize * 8;

                // Collect the data bits.
                let mut data_bits = BitVec::new();
                for _ in 0..bit_length {
                    data_bits.push(bit_stream.next().ok_or(
                        CalldataElementAPEDecodeError::Bytes(
                            BytesAPEDecodeError::CollectDataBitsError,
                        ),
                    )?);
                }

                // Convert the bits to data bytes.
                let data_bytes = data_bits.to_bytes();

                // Construct the `CalldataElement`.
                let element = CalldataElement::Bytes(data_bytes);

                // Return the element.
                Ok(element)
            }

            // Decode the `Varbytes`.
            CalldataElementType::Varbytes => {
                // Initialize a bit vector to fill with byte length.
                let mut byte_length_bits = BitVec::new();

                // Collect 12 bits representing the byte length.
                // Supported byte-length range: 0 to 4095.
                for _ in 0..12 {
                    byte_length_bits.push(bit_stream.next().ok_or(
                        CalldataElementAPEDecodeError::Varbytes(
                            VarbytesAPEDecodeError::CollectVarbytesLengthBitsError,
                        ),
                    )?);
                }

                // Convert the byte length bits to a u16.
                let byte_length = convert_12_bits_to_u16(&byte_length_bits);

                // Return an error if the byte length is greater than 4095.
                if byte_length > 4095 {
                    return Err(CalldataElementAPEDecodeError::Varbytes(
                        VarbytesAPEDecodeError::ByteLengthGreaterThan4095Error(byte_length),
                    ));
                }

                // If the data length is 0, return an empty `Varbytes`.
                if byte_length == 0 {
                    return Ok(CalldataElement::Varbytes(vec![]));
                }

                // Convert to bit length.
                let bit_length = byte_length as usize * 8;

                // Initialize bit vector to fill with data.
                let mut data_bits = BitVec::new();

                // Collect the data bit by bit.
                for _ in 0..bit_length {
                    data_bits.push(bit_stream.next().ok_or(
                        CalldataElementAPEDecodeError::Varbytes(
                            VarbytesAPEDecodeError::CollectVarbytesDataBitsError,
                        ),
                    )?);
                }

                // Convert the bits to bytes.
                let data_bytes = data_bits.to_bytes();

                // Construct `CalldataElement` from the bytes.
                let element = CalldataElement::Varbytes(data_bytes);

                // Return the element.
                Ok(element)
            }

            // Decode the `Payable`.
            CalldataElementType::Payable => {
                // Decode the `ShortVal` from `MaybeCommon<ShortVal>`.
                let short_val = MaybeCommon::<ShortVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CalldataElementAPEDecodeError::Payable(
                            PayableAPEDecodeError::MaybeCommonShortValAPEDecodingError(e),
                        )
                    })?
                    .value();

                // Construct the `CalldataElement`.
                let element = CalldataElement::Payable(short_val);

                // Return the element.
                Ok(element)
            }
        }
    }
}

/// Converts 12 bits to a u16.
fn convert_12_bits_to_u16(bits: &BitVec) -> u16 {
    // Initialize a u16 value.
    let mut byte_length = 0u16;

    // Iterate over 12 bits.
    for i in 0..12 {
        let bit = bits[i];
        if bit {
            byte_length |= 1 << i;
        }
    }

    // Return the u16 value.
    byte_length
}
