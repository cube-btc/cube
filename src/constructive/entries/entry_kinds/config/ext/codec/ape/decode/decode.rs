use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::codec::ape::decode::error::decode_error::ConfigAPEDecodeError;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::inscriptive::registry::registry::REGISTRY;
use bit_vec::BitVec;

impl Config {
    /// Decodes a `Config` as an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        decode_account_rank_as_longval: bool,
        registry: &REGISTRY,
    ) -> Result<Config, ConfigAPEDecodeError> {
        let root_account = RootAccount::decode_ape(bit_stream, decode_account_rank_as_longval, registry)
            .await
            .map_err(ConfigAPEDecodeError::RootAccountAPEDecodeError)?;

        let secondary_aggregation_key_present = bit_stream
            .next()
            .ok_or(ConfigAPEDecodeError::SecondaryAggregationKeyPresenceBitCollectError)?;
        let secondary_aggregation_key = if secondary_aggregation_key_present {
            let key_len = ShortVal::decode_ape(bit_stream)
                .map_err(ConfigAPEDecodeError::SecondaryAggregationKeyLenDecodeError)?
                .value() as usize;
            let key_bits: BitVec = bit_stream.by_ref().take(key_len * 8).collect();
            if key_bits.len() != key_len * 8 {
                return Err(ConfigAPEDecodeError::SecondaryAggregationKeyBitsCollectError);
            }
            Some(key_bits.to_bytes())
        } else {
            None
        };

        let projector_config_present = bit_stream
            .next()
            .ok_or(ConfigAPEDecodeError::ProjectorConfigPresenceBitCollectError)?;
        let projector_config = if projector_config_present {
            let projector_bits: BitVec = bit_stream.by_ref().take(256).collect();
            if projector_bits.len() != 256 {
                return Err(ConfigAPEDecodeError::ProjectorConfigBitsCollectError);
            }
            let projector_bytes = projector_bits.to_bytes();
            Some(
                projector_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| ConfigAPEDecodeError::ProjectorConfigBytesConversionError)?,
            )
        } else {
            None
        };

        let flame_config_present = bit_stream
            .next()
            .ok_or(ConfigAPEDecodeError::FlameConfigPresenceBitCollectError)?;
        let flame_config = if flame_config_present {
            let flame_len = ShortVal::decode_ape(bit_stream)
                .map_err(ConfigAPEDecodeError::FlameConfigLenDecodeError)?
                .value() as usize;
            let flame_bits: BitVec = bit_stream.by_ref().take(flame_len * 8).collect();
            if flame_bits.len() != flame_len * 8 {
                return Err(ConfigAPEDecodeError::FlameConfigBitsCollectError);
            }
            FMAccountFlameConfig::from_bytes(&flame_bits.to_bytes())
                .ok_or(ConfigAPEDecodeError::FlameConfigDecodeError)
                .map(Some)?
        } else {
            None
        };

        let target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(ConfigAPEDecodeError::TargetAPEDecodeError)?;

        Ok(Config::new(
            root_account,
            secondary_aggregation_key,
            projector_config,
            flame_config,
            target,
        ))
    }
}
