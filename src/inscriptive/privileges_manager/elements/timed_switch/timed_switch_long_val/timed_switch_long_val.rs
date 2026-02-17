use serde::{Deserialize, Serialize};

/// Represents a precedence value for a long val.
type LongValPrecedence = u64;

/// Represents the timeout duration after which the long val switches back to the default value.
type Timeout = u64;

/// Represents a dual-value long val (u64) that switches back to the default value after timeout.
#[derive(Clone, Serialize, Deserialize)]
pub struct TimedSwitchLongVal {
    pub default_value: u64,
    pub precedence: Option<(LongValPrecedence, Timeout)>,
}

impl TimedSwitchLongVal {
    /// Creates a new `TimedSwitchLongVal` with the given parameters.
    pub fn new(default_value: u64, precedence: Option<(LongValPrecedence, Timeout)>) -> Self {
        Self {
            default_value,
            precedence,
        }
    }

    /// Returns the value of the long val at the given timestamp.
    pub fn get_value(&self, current_timestamp: u64) -> u64 {
        // 1 Check if there is a precedence.
        match self.precedence {
            // 1.a Precedence is set.
            Some((precedence_value, timeout)) => {
                // 1.a.1 Check if the time is out.
                match current_timestamp >= timeout {
                    // 1.a.1.1 Timeout case. Back to the default value.
                    true => self.default_value,
                    // 1.a.1.2 No timeout. Use the precedence value.
                    false => precedence_value,
                }
            }
            // 1.b Precedence is not set.
            None => self.default_value,
        }
    }

    /// Updates the default value.
    pub fn update_default_value(&mut self, default_value: u64) {
        self.default_value = default_value;
    }

    /// Updates the precedence.
    pub fn update_precedence(&mut self, precedence: Option<(LongValPrecedence, Timeout)>) {
        self.precedence = precedence;
    }

    /// Serializes the timed switch long val to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::new();

        // 2 Serialize the default value.
        bytes.extend(self.default_value.to_le_bytes());

        // 3 Serialize the precedence byte(s).
        match self.precedence {
            Some((precedence_value, timeout)) => {
                bytes.push(0x01); // Precedence is set.
                bytes.extend(precedence_value.to_le_bytes());
                bytes.extend(timeout.to_le_bytes());
            }
            None => {
                bytes.push(0x00); // Precedence is not set.
            }
        }

        // 4 Return the bytes.
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<TimedSwitchLongVal> {
        // 1 Match on the byte vector length.
        match bytes.len() {
            // 1.a It's default-value-only if the byte vector length is 9.
            9 => {
                // 1.a.1 Deserialize the default value.
                let default_value = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

                // 1.a.2 Deserialize the precedence byte.
                let precedence_byte = u8::from_le_bytes(bytes[8..9].try_into().ok()?);

                // 1.a.3 Check if the precedence byte is valid.
                if precedence_byte != 0x00 {
                    return None; // Invalid precedence byte.
                }

                // 1.a.4 Construct the timed switch long val.
                let timed_switch_long_val = TimedSwitchLongVal::new(default_value, None);

                // 1.a.5 Return the timed switch long val.
                Some(timed_switch_long_val)
            }
            // 1.b It's default value + precedence if the byte vector length is 25.
            25 => {
                // 1.b.1 Deserialize the default value.
                let default_value = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

                // 1.b.2 Deserialize the precedence byte.
                let precedence_byte = u8::from_le_bytes(bytes[8..9].try_into().ok()?);

                // 1.b.3 Check if the precedence byte is valid.
                if precedence_byte != 0x01 {
                    return None; // Invalid precedence byte.
                }

                // 1.b.4 Deserialize the precedence.
                let precedence = Some((
                    u64::from_le_bytes(bytes[9..17].try_into().ok()?),
                    u64::from_le_bytes(bytes[17..25].try_into().ok()?),
                ));

                // 1.b.5 Construct the timed switch long val.
                let timed_switch_long_val = TimedSwitchLongVal::new(default_value, precedence);

                // 1.b.6 Return the timed switch long val. Some(timed_switch_long_val)
                Some(timed_switch_long_val)
            }
            // 1.c Invalid length.
            _ => None,
        }
    }
}
