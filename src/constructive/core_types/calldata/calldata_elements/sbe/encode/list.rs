use crate::constructive::calldata::element::calldata_element::CalldataElement;
use crate::constructive::calldata::element::sbe::encode::error::encode_error::CalldataElementSBEEncodeError;

type Bytes = Vec<u8>;

/// Encodes a calldata element list as SBE bytes.
///
/// Layout: `u32` little-endian element count, then each element's [`CalldataElement::encode_sbe`] payload.
pub fn encode_calldata_elements_sbe(
    elements: &[CalldataElement],
) -> Result<Bytes, CalldataElementSBEEncodeError> {
    // 1 Encode each element payload.
    let mut element_payloads = Vec::with_capacity(elements.len());
    for element in elements {
        element_payloads.push(element.encode_sbe()?);
    }

    // 2 Ensure element count fits `u32`.
    let count_u32 = u32::try_from(elements.len()).map_err(|_| {
        CalldataElementSBEEncodeError::ElementCountTooLargeForU32 {
            len: elements.len(),
        }
    })?;

    // 3 Initialize bytes and write layout.
    let mut bytes = Bytes::new();
    bytes.extend_from_slice(&count_u32.to_le_bytes());
    for payload in element_payloads {
        bytes.extend(payload);
    }

    // 4 Return bytes.
    Ok(bytes)
}
