use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::codec::sbe::encode::error::encode_error::ConfigSBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Config {
    /// Structural Byte-scope Encoding (SBE) encoding for `Config`.
    pub fn encode_sbe(&self) -> Result<Bytes, ConfigSBEEncodeError> {
        let root_account_bytes = self.root_account.encode_sbe();
        let root_len_u32 = u32::try_from(root_account_bytes.len()).map_err(|_| {
            ConfigSBEEncodeError::ConfigSBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: root_account_bytes.len(),
            }
        })?;

        let secondary_key_bytes = self.secondary_aggregation_key.clone().unwrap_or_default();
        let secondary_key_len_u32 = u32::try_from(secondary_key_bytes.len()).map_err(|_| {
            ConfigSBEEncodeError::ConfigSBESecondaryAggregationKeyPayloadTooLargeForU32LengthPrefix {
                len: secondary_key_bytes.len(),
            }
        })?;

        let flame_config_bytes = self
            .flame_config
            .as_ref()
            .map(|cfg| cfg.to_bytes())
            .unwrap_or_default();
        let flame_len_u32 = u32::try_from(flame_config_bytes.len()).map_err(|_| {
            ConfigSBEEncodeError::ConfigSBEFlameConfigPayloadTooLargeForU32LengthPrefix {
                len: flame_config_bytes.len(),
            }
        })?;

        let mut bytes = Bytes::new();
        bytes.push(0x07);

        bytes.extend_from_slice(&root_len_u32.to_le_bytes());
        bytes.extend_from_slice(&root_account_bytes);

        bytes.push(self.secondary_aggregation_key.is_some() as u8);
        bytes.extend_from_slice(&secondary_key_len_u32.to_le_bytes());
        bytes.extend_from_slice(&secondary_key_bytes);

        bytes.push(self.projector_config.is_some() as u8);
        if let Some(projector_config) = self.projector_config {
            bytes.extend_from_slice(&projector_config);
        }

        bytes.push(self.flame_config.is_some() as u8);
        bytes.extend_from_slice(&flame_len_u32.to_le_bytes());
        bytes.extend_from_slice(&flame_config_bytes);

        bytes.extend_from_slice(&self.target.encode_sbe());

        Ok(bytes)
    }
}
