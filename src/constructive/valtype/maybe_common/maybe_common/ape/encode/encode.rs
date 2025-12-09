use crate::constructive::valtype::maybe_common::maybe_common::ape::encode::error::encode_error::MaybeCommonAPEEncodeError;
use crate::constructive::valtype::{
    maybe_common::maybe_common::maybe_common::{
        CommonVal, Commonable, MaybeCommon, MaybeCommonValue,
    },
    val::{long_val::long_val::LongVal, short_val::short_val::ShortVal},
};
use bit_vec::BitVec;

impl<T> MaybeCommon<T>
where
    T: Commonable + Clone + From<ShortVal> + From<LongVal>,
{
    /// Airly Payload Encoding (APE) encoding for `MaybeCommon`.
    ///
    /// This function encodes a `MaybeCommon` into an Airly Payload Encoding (APE) bit vector.
    /// The `MaybeCommon` can be a `CommonVal` or a `UncommonVal`.
    ///
    /// # Arguments
    /// * `&self` - The `MaybeCommon` to encode.
    pub fn encode_ape(&self) -> Result<BitVec, MaybeCommonAPEEncodeError> {
        // 1 Create a new BitVec.
        let mut bits = BitVec::new();

        // 2 Match on whether the `MaybeCommon` is a `CommonVal` or a `UncommonVal`.
        match self {
            // 2.a The `MaybeCommon` is a `CommonVal`.
            MaybeCommon::Common(common_val) => {
                // 2.a.1 Insert true for common.
                bits.push(true);

                // 2.a.2 Insert the common value.
                match common_val {
                    // 2.a.2.a The `CommonVal` is a `CommonShortVal`.
                    CommonVal::CommonShort(common_short_val) => {
                        // 2.a.2.a.1 Extend 6 bits.
                        bits.extend(common_short_val.encode_ape().map_err(|e| {
                            MaybeCommonAPEEncodeError::CommonShortValAPEEncodeError(e)
                        })?);
                    }
                    // 2.a.2.b The `CommonVal` is a `CommonLongVal`.
                    CommonVal::CommonLong(common_long_val) => {
                        // 2.a.2.b.1 Extend 7 bits.
                        bits.extend(common_long_val.encode_ape().map_err(|e| {
                            MaybeCommonAPEEncodeError::CommonLongValAPEEncodeError(e)
                        })?);
                    }
                }

                // 2.3 Return the bits.
                Ok(bits)
            }

            // 2.b The `MaybeCommon` is a `UncommonVal`.
            MaybeCommon::Uncommon(uncommon_val) => {
                // 2.4 Insert false for uncommon.
                bits.push(false);

                // 2.5 Encode the uncommon value based on its type
                match uncommon_val.maybe_common_value() {
                    // 2.5.a The `UncommonVal` is a `ShortVal`.
                    MaybeCommonValue::Short(short_val) => {
                        // 2.5.a.1 Encode as ShortVal
                        bits.extend(short_val.encode_ape());
                    }

                    // 2.5.b The `UncommonVal` is a `LongVal`.
                    MaybeCommonValue::Long(long_val) => {
                        // 2.5.b.1 Encode as LongVal
                        bits.extend(long_val.encode_ape());
                    }
                }

                // 2.6 Return the bit vector.
                Ok(bits)
            }
        }
    }
}
