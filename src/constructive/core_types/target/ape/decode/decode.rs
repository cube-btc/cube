use crate::constructive::core_types::target::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::core_types::target::target::Target;

/// Airly Payload Encoding (APE) decoding for `Target`.
impl Target {
    /// Decodes a `Target` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// This function decodes a `Target` from an Airly Payload Encoding (APE) bit stream.
    /// A leading `false` bit means full overlap with `execution_batch_height`. A leading `true`
    /// bit means the next two bits encode a batch gap (1..=4) subtracted from execution height.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        execution_batch_height: u64,
    ) -> Result<Target, TargetAPEDecodeError> {
        // 1 Read the leading bit (`false` means full overlap with `execution_batch_height`).
        let has_batch_gap = bit_stream
            .next()
            .ok_or(TargetAPEDecodeError::UnexpectedEndOfBitstream)?;

        // 2 If there is no gap prefix, the target height equals the execution height.
        if !has_batch_gap {
            return Ok(Target::new(execution_batch_height));
        }

        // 3 Read the two gap bits (MSB first).
        let b0 = bit_stream
            .next()
            .ok_or(TargetAPEDecodeError::UnexpectedEndOfBitstream)?;
        let b1 = bit_stream
            .next()
            .ok_or(TargetAPEDecodeError::UnexpectedEndOfBitstream)?;

        // 4 Compute `targeted_at_batch_height` from the gap, or fail on underflow.
        let gap_minus_one = u64::from(b0 as u8) * 2 + u64::from(b1 as u8);
        let gap = gap_minus_one + 1;
        let targeted_at_batch_height = execution_batch_height
            .checked_sub(gap)
            .ok_or(TargetAPEDecodeError::TargetBatchHeightUnderflow)?;

        // 5 Construct and return the `Target`.
        Ok(Target::new(targeted_at_batch_height))
    }
}
