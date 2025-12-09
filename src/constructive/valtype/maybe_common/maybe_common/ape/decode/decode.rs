use crate::constructive::valtype::maybe_common::common::common_long::common_long::CommonLongVal;
use crate::constructive::valtype::maybe_common::common::common_short::common_short::CommonShortVal;
use crate::constructive::valtype::maybe_common::maybe_common::ape::decode::error::decode_error::MaybeCommonAPEDecodeError;
use crate::constructive::valtype::maybe_common::maybe_common::maybe_common::MaybeCommonValueType;
use crate::constructive::valtype::maybe_common::maybe_common::maybe_common::{
    CommonVal, Commonable, MaybeCommon,
};
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;

impl<T> MaybeCommon<T>
where
    T: Commonable + Clone + From<ShortVal> + From<LongVal>,
{
    /// Airly Payload Encoding (APE) decoding for `MaybeCommon`.
    ///
    /// This function decodes a `MaybeCommon` from an Airly Payload Encoding (APE) bit stream.
    /// The `MaybeCommon` can be a `CommonVal` or a `UncommonVal`.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<MaybeCommon<T>, MaybeCommonAPEDecodeError> {
        // 1 Check if the value is common.
        let is_common = bit_stream
            .next()
            .ok_or(MaybeCommonAPEDecodeError::IsCommonBitCollectError)?;

        // 2 Match on whether the `MaybeCommon` is a `CommonVal` or a `UncommonVal`.
        match is_common {
            // 2.a The `MaybeCommon` is a `CommonVal`.
            true => {
                // 2.a.1 Value is common.

                // 2.a.2 Check if the common value is short or long
                match T::maybe_common_value_type() {
                    MaybeCommonValueType::Short => {
                        // 2.a.2.a Decode common short value from 6 bits.
                        let common_short_val =
                            CommonShortVal::decode_ape(bit_stream).map_err(|e| {
                                MaybeCommonAPEDecodeError::CommonShortValAPEDecodeError(e)
                            })?;

                        // 2.a.2.a.1 Return the common short value.
                        Ok(MaybeCommon::Common(CommonVal::CommonShort(
                            common_short_val,
                        )))
                    }
                    MaybeCommonValueType::Long => {
                        // 2.a.2.b Decode common long value from 7 bits.
                        let common_long_val =
                            CommonLongVal::decode_ape(bit_stream).map_err(|e| {
                                MaybeCommonAPEDecodeError::CommonLongValAPEDecodeError(e)
                            })?;

                        // 2.a.2.b.1 Return the common long value.
                        Ok(MaybeCommon::Common(CommonVal::CommonLong(common_long_val)))
                    }
                }
            }

            // 2.b The `MaybeCommon` is a `UncommonVal`.
            false => {
                // 2.b.1 Check if the uncommon value is short or long
                match T::maybe_common_value_type() {
                    // 2.b.1.a The `UncommonVal` is a `ShortVal`.
                    MaybeCommonValueType::Short => {
                        // 2.b.1.a.1 Decode uncommon short value.
                        let uncommon_short_val = ShortVal::decode_ape(bit_stream)
                            .map_err(|e| MaybeCommonAPEDecodeError::ShortValAPEDecodeError(e))?;

                        // 2.b.1.a.2 Return the uncommon short value.
                        Ok(MaybeCommon::Uncommon(uncommon_short_val.into()))
                    }

                    // 2.b.1.b The `UncommonVal` is a `LongVal`.
                    MaybeCommonValueType::Long => {
                        // 2.b.1.b.1 Decode uncommon long value.
                        let uncommon_long_val = LongVal::decode_ape(bit_stream)
                            .map_err(|e| MaybeCommonAPEDecodeError::LongValAPEDecodeError(e))?;

                        // 2.b.1.b.2 Return the uncommon long value.
                        Ok(MaybeCommon::Uncommon(uncommon_long_val.into()))
                    }
                }
            }
        }
    }
}
