use crate::constructive::calldata::element::calldata_element::CalldataElement;
use crate::constructive::calldata::element::sbe::decode::error::decode_errors::CalldataElementsSBEDecodeError;

/// Decodes a calldata element list from SBE bytes produced by [`encode_calldata_elements_sbe`](crate::constructive::calldata::element::sbe::encode::list::encode_calldata_elements_sbe).
pub fn decode_calldata_elements_sbe(
    bytes: &[u8],
) -> Result<Vec<CalldataElement>, CalldataElementsSBEDecodeError> {
    if bytes.len() < 4 {
        return Err(CalldataElementsSBEDecodeError::InsufficientBytesForElementCount {
            got: bytes.len(),
        });
    }

    let count = u32::from_le_bytes(
        bytes[0..4]
            .try_into()
            .map_err(|_| CalldataElementsSBEDecodeError::ElementCountBytesConversionError)?,
    ) as usize;

    let mut rest = &bytes[4..];
    let mut elements = Vec::with_capacity(count);

    for _ in 0..count {
        let (element, remaining) = CalldataElement::decode_sbe(rest).map_err(|e| {
            CalldataElementsSBEDecodeError::Element(e)
        })?;
        elements.push(element);
        rest = remaining;
    }

    if !rest.is_empty() {
        return Err(CalldataElementsSBEDecodeError::TrailingBytesAfterCalldataList {
            trailing: rest.len(),
        });
    }

    Ok(elements)
}
