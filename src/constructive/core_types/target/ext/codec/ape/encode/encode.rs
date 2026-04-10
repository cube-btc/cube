use crate::constructive::core_types::target::ext::codec::ape::encode::error::encode_error::TargetAPEEncodeError;
use crate::constructive::core_types::target::target::Target;
use bit_vec::BitVec;

impl Target {
    /// Encodes a `Target` into an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes a `Target` into an Airly Payload Encoding (APE) bit vector.
    /// When `execution_batch_height` equals `targeted_at_batch_height`, a single `false` bit
    /// indicates full overlap. Otherwise a `true` bit and two further bits encode the batch gap
    /// between execution and target heights in the range 1..=4 (`00`..=`11` map to gaps 1..=4).
    pub fn encode_ape(
        &self,
        execution_batch_height: u64,
    ) -> Result<BitVec, TargetAPEEncodeError> {
        // 1 Get `targeted_at_batch_height` from `self`.
        let targeted = self.targeted_at_batch_height;

        // 2 Compute the gap from execution height to targeted height, or fail if the target is after execution.
        let gap = execution_batch_height
            .checked_sub(targeted)
            .ok_or(TargetAPEEncodeError::TargetAfterExecution)?;

        // 3 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 4 Match on the gap and append bits.
        match gap {
            // 4.a Full overlap: one `false` bit.
            0 => bits.push(false),
            // 4.b Representable gap: `true` then two bits for `gap - 1` (MSB first).
            1..=4 => {
                bits.push(true);
                let encoded = gap - 1;
                bits.push(encoded & 2 != 0);
                bits.push(encoded & 1 != 0);
            }
            // 4.c Gap larger than four batches: encoding is not representable.
            _ => return Err(TargetAPEEncodeError::TargetTooFarInPast),
        }

        // 5 Return the bit vector.
        Ok(bits)
    }
}
