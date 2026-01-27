use serde::{Deserialize, Serialize};

/// The timestamp when the freeze or destroy action takes effect.
type TakesEffectAtTimestamp = u64;

/// The liveness state of an account/contract.
#[derive(Clone, Serialize, Deserialize)]
pub enum LivenessFlag {
    Operational,

    // The account/contract will be frozen.
    ToBeFrozen(TakesEffectAtTimestamp),

    // The account/contract will be destroyed.
    ToBeDestroyed(TakesEffectAtTimestamp),
}

impl LivenessFlag {
    /// Creates a new liveness flag.
    pub fn new_operational() -> Self {
        Self::Operational
    }

    /// Creates a new to be frozen liveness flag.
    pub fn new_to_be_frozen(timestamp: TakesEffectAtTimestamp) -> Self {
        Self::ToBeFrozen(timestamp)
    }

    /// Creates a new to be destroyed liveness flag.
    pub fn new_to_be_destroyed(timestamp: TakesEffectAtTimestamp) -> Self {
        Self::ToBeDestroyed(timestamp)
    }

    /// Returns whether the liveness flag is operational.
    pub fn is_operational(&self, current_timestamp: u64) -> bool {
        // 1 Match on the liveness flag.
        match self {
            // 1.a If the liveness flag is operational.
            Self::Operational => true,

            // 1.b If the liveness flag is to be frozen.
            Self::ToBeFrozen(takes_effect_at_timestamp) => {
                match current_timestamp >= *takes_effect_at_timestamp {
                    true => false,
                    false => true,
                }
            }

            // 1.c If the liveness flag is to be destroyed.
            Self::ToBeDestroyed(takes_effect_at_timestamp) => {
                match current_timestamp >= *takes_effect_at_timestamp {
                    true => false,
                    false => true,
                }
            }
        }
    }

    /// Returns whether the account or contract should be destroyed.
    pub fn should_be_destroyed(&self, current_timestamp: u64) -> bool {
        // 1 Match on the liveness flag.
        match self {
            // 1.a Account/contract is to be destroyed.
            Self::ToBeDestroyed(takes_effect_at_timestamp) => {
                match current_timestamp >= *takes_effect_at_timestamp {
                    // 1.a.1 Must be destroyed now.
                    true => true,

                    // 1.a.2 Not yet, but soon.
                    false => false,
                }
            }

            // 1.b Account/contract is operational or to be frozen.
            _ => false,
        }
    }

    /// Returns the timestamp when the freeze or destroy action takes effect.
    pub fn takes_effect_at_timestamp(&self) -> Option<TakesEffectAtTimestamp> {
        match self {
            Self::ToBeFrozen(timestamp) => Some(*timestamp),
            Self::ToBeDestroyed(timestamp) => Some(*timestamp),
            _ => None,
        }
    }

    /// Serializes the liveness flag to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Match on the liveness flag.
        match self {
            // 1.a If the liveness flag is operational.
            Self::Operational => vec![0x00],

            // 1.b If the liveness flag is to be frozen.
            Self::ToBeFrozen(timestamp) => {
                // 1.b.1 Create a new vector of bytes.
                let mut bytes = Vec::<u8>::new();

                // 1.b.2 Push the bytecode for the liveness flag.
                bytes.push(0x01);

                // 1.b.3 Push the timestamp.
                bytes.extend_from_slice(&timestamp.to_le_bytes());

                // 1.b.4 Return the bytes.
                bytes
            }

            // 1.c If the liveness flag is to be destroyed.
            Self::ToBeDestroyed(timestamp) => {
                // 1.c.1 Create a new vector of bytes.
                let mut bytes = Vec::<u8>::new();

                // 1.c.2 Push the bytecode for the liveness flag.
                bytes.push(0x02);

                // 1.c.3 Push the timestamp.
                bytes.extend_from_slice(&timestamp.to_le_bytes());

                // 1.c.4 Return the bytes.
                bytes
            }
        }
    }

    /// Deserializes the liveness flag from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<LivenessFlag> {
        // 1 Check if there are enough bytes for the liveness flag.
        if bytes.len() < 1 {
            return None;
        }

        // 2 Match on the liveness flag.
        match bytes[0] {
            // 2.a If the liveness flag is operational.
            0x00 => Some(Self::Operational),

            // 2.b If the liveness flag is to be frozen.
            0x01 => Some(Self::ToBeFrozen(TakesEffectAtTimestamp::from_le_bytes(
                bytes[1..9].try_into().ok()?,
            ))),

            // 2.c If the liveness flag is to be destroyed.
            0x02 => Some(Self::ToBeDestroyed(TakesEffectAtTimestamp::from_le_bytes(
                bytes[1..9].try_into().ok()?,
            ))),

            // 2.d Otherwise, return None.
            _ => None,
        }
    }
}
