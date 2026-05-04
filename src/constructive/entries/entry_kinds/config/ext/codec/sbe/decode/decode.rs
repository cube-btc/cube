use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::codec::sbe::decode::error::decode_error::ConfigSBEDecodeError;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;

impl Config {
    /// Decodes a `Config` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Config, ConfigSBEDecodeError> {
        if bytes.is_empty() {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: 0,
            });
        }
        if bytes[0] != 0x07 {
            return Err(ConfigSBEDecodeError::InvalidEntryKindByteError {
                expected: 0x07,
                got: bytes[0],
            });
        }

        if bytes.len() < 5 {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let root_len = u32::from_le_bytes(
            bytes[1..5]
                .try_into()
                .map_err(|_| ConfigSBEDecodeError::ConfigSBERootAccountLengthPrefixBytesConversionError)?,
        ) as usize;
        let after_root_len_prefix = &bytes[5..];
        if after_root_len_prefix.len() < root_len {
            return Err(ConfigSBEDecodeError::ConfigSBERootAccountLengthPrefixExceedsPayload {
                root_len,
                got_after_prefix: after_root_len_prefix.len(),
            });
        }
        let (root_slice, mut tail) = after_root_len_prefix.split_at(root_len);
        let root_account = RootAccount::decode_sbe(root_slice)
            .map_err(ConfigSBEDecodeError::ConfigSBERootAccountDecodeError)?;

        if tail.is_empty() {
            return Err(
                ConfigSBEDecodeError::ConfigSBEInsufficientBytesForSecondaryAggregationPresenceFlag {
                    got_total: bytes.len(),
                },
            );
        }
        let secondary_present = tail[0] != 0;
        tail = &tail[1..];

        if tail.len() < 4 {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForSecondaryAggregationLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let secondary_len = u32::from_le_bytes(
            tail[0..4]
                .try_into()
                .map_err(|_| ConfigSBEDecodeError::ConfigSBESecondaryAggregationLengthPrefixBytesConversionError)?,
        ) as usize;
        tail = &tail[4..];
        if tail.len() < secondary_len {
            return Err(ConfigSBEDecodeError::ConfigSBESecondaryAggregationLengthPrefixExceedsPayload {
                key_len: secondary_len,
                got_after_prefix: tail.len(),
            });
        }
        let (secondary_slice, tail_after_secondary) = tail.split_at(secondary_len);
        let secondary_aggregation_key = if secondary_present {
            Some(secondary_slice.to_vec())
        } else {
            if secondary_len != 0 {
                return Err(
                    ConfigSBEDecodeError::ConfigSBESecondaryAggregationPresenceLengthMismatch {
                        key_len: secondary_len,
                    },
                );
            }
            None
        };
        tail = tail_after_secondary;

        if tail.is_empty() {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForProjectorConfigPresenceFlag {
                got_total: bytes.len(),
            });
        }
        let projector_present = tail[0] != 0;
        tail = &tail[1..];
        let projector_config = if projector_present {
            if tail.len() < 32 {
                return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForProjectorConfigPayload {
                    got_total: bytes.len(),
                });
            }
            let projector_bytes: [u8; 32] = tail[0..32]
                .try_into()
                .map_err(|_| ConfigSBEDecodeError::ConfigSBEProjectorConfigPresenceMismatch)?;
            tail = &tail[32..];
            Some(projector_bytes)
        } else {
            None
        };

        if tail.is_empty() {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForFlameConfigPresenceFlag {
                got_total: bytes.len(),
            });
        }
        let flame_present = tail[0] != 0;
        tail = &tail[1..];

        if tail.len() < 4 {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForFlameConfigLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let flame_len = u32::from_le_bytes(
            tail[0..4]
                .try_into()
                .map_err(|_| ConfigSBEDecodeError::ConfigSBEFlameConfigLengthPrefixBytesConversionError)?,
        ) as usize;
        tail = &tail[4..];
        if tail.len() < flame_len {
            return Err(ConfigSBEDecodeError::ConfigSBEFlameConfigLengthPrefixExceedsPayload {
                flame_len,
                got_after_prefix: tail.len(),
            });
        }
        let (flame_slice, tail_after_flame) = tail.split_at(flame_len);
        let flame_config = if flame_present {
            FMAccountFlameConfig::from_bytes(flame_slice)
                .ok_or(ConfigSBEDecodeError::ConfigSBEFlameConfigDecodeError)
                .map(Some)?
        } else {
            if flame_len != 0 {
                return Err(ConfigSBEDecodeError::ConfigSBEFlameConfigPresenceLengthMismatch {
                    flame_len,
                });
            }
            None
        };
        tail = tail_after_flame;

        if tail.len() < 8 {
            return Err(ConfigSBEDecodeError::ConfigSBEInsufficientBytesForTarget {
                got_total: bytes.len(),
            });
        }
        let target = Target::decode_sbe(&tail[0..8])
            .map_err(ConfigSBEDecodeError::ConfigSBETargetDecodeError)?;
        tail = &tail[8..];

        if !tail.is_empty() {
            return Err(ConfigSBEDecodeError::ConfigSBETrailingBytesAfterConfig {
                trailing: tail.len(),
            });
        }

        Ok(Config::new(
            root_account,
            secondary_aggregation_key,
            projector_config,
            flame_config,
            target,
        ))
    }
}
