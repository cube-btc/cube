use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::codec::ape::encode::error::encode_error::ConfigAPEEncodeError;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registry::registry::REGISTRY;
use bit_vec::BitVec;

impl Config {
    /// Airly Payload Encoding (APE) encoding for `Config`.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registry: &REGISTRY,
        encode_account_rank_as_longval: bool,
    ) -> Result<BitVec, ConfigAPEEncodeError> {
        let mut bits = BitVec::new();

        let root_bits = self
            .root_account
            .encode_ape(registry, encode_account_rank_as_longval)
            .await
            .map_err(ConfigAPEEncodeError::RootAccountAPEEncodeError)?;
        bits.extend(root_bits);

        match &self.secondary_aggregation_key {
            Some(key) => {
                bits.push(true);
                let key_len = u32::try_from(key.len())
                    .map_err(|_| ConfigAPEEncodeError::SecondaryAggregationKeyLenTooLarge(key.len()))?;
                bits.extend(ShortVal::new(key_len).encode_ape());
                bits.extend(BitVec::from_bytes(key));
            }
            None => bits.push(false),
        }

        match self.projector_config {
            Some(projector_config) => {
                bits.push(true);
                bits.extend(BitVec::from_bytes(&projector_config));
            }
            None => bits.push(false),
        }

        match &self.flame_config {
            Some(flame_config) => {
                bits.push(true);
                let flame_config_bytes = flame_config.to_bytes();
                let flame_len = u32::try_from(flame_config_bytes.len())
                    .map_err(|_| ConfigAPEEncodeError::FlameConfigLenTooLarge(flame_config_bytes.len()))?;
                bits.extend(ShortVal::new(flame_len).encode_ape());
                bits.extend(BitVec::from_bytes(&flame_config_bytes));
            }
            None => bits.push(false),
        }

        let target_bits = self
            .target
            .encode_ape(execution_batch_height)
            .map_err(ConfigAPEEncodeError::TargetAPEEncodeError)?;
        bits.extend(target_bits);

        Ok(bits)
    }
}
