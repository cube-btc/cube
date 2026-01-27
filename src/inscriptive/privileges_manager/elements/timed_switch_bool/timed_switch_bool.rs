use serde::{Deserialize, Serialize};

/// Represents a precedence value for a bool.
type BoolPrecedence = bool;

/// Represents the timeout duration after which the bool switches back to the default value.
type Timeout = u64;

/// Represents a dual-value bool that switches back to the default value after timeout.
#[derive(Clone, Serialize, Deserialize)]
pub struct TimedSwitchBool {
    pub default_value: bool,
    pub precedence: Option<(BoolPrecedence, Timeout)>,
}

impl TimedSwitchBool {
    /// Creates a new `TimedSwitchBool` with the given parameters.
    pub fn new(default_value: bool, precedence: Option<(BoolPrecedence, Timeout)>) -> Self {
        Self {
            default_value,
            precedence,
        }
    }

    /// Returns the value of the bool at the given timestamp.
    pub fn get_value(&self, current_timestamp: u64) -> bool {
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
    pub fn update_default_value(&mut self, default_value: bool) {
        self.default_value = default_value;
    }

    /// Updates the precedence.
    pub fn update_precedence(&mut self, precedence: Option<(BoolPrecedence, Timeout)>) {
        self.precedence = precedence;
    }

    /// Serializes the timed switch bool to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::new();

        // 2 Serialize the default value.
        bytes.push(self.default_value as u8);

        // 3 Serialize the precedence byte(s).
        match self.precedence {
            Some((precedence_value, timeout)) => {
                bytes.push(0x01); // Precedence is set.
                bytes.push(precedence_value as u8);
                bytes.extend(timeout.to_le_bytes());
            }
            None => {
                bytes.push(0x00); // Precedence is not set.
            }
        }

        // 4 Return the bytes.
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<TimedSwitchBool> {
        // 1 Match on the byte vector length.
        match bytes.len() {
            // 1.a It's default-value-only if the byte vector length is 2.
            2 => {
                // 1.a.1 Deserialize the default value.
                let default_value = match u8::from_le_bytes(bytes[0..1].try_into().ok()?) {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                // 1.a.2 Deserialize the precedence byte.
                let precedence_byte = u8::from_le_bytes(bytes[1..2].try_into().ok()?);

                // 1.a.3 Check if the precedence byte is valid.
                if precedence_byte != 0x00 {
                    return None; // Invalid precedence byte.
                }

                // 1.a.4 Construct the timed switch bool.
                let timed_switch_bool = TimedSwitchBool::new(default_value, None);

                // 1.a.5 Return the timed switch bool.
                Some(timed_switch_bool)
            }
            // 1.b It's default value + precedence if the byte vector length is 11.
            11 => {
                // 1.b.1 Deserialize the default value.
                let default_value = match u8::from_le_bytes(bytes[0..1].try_into().ok()?) {
                    0 => false,
                    1 => true,
                    _ => return None,
                };

                // 1.b.2 Deserialize the precedence byte.
                let precedence_byte = u8::from_le_bytes(bytes[1..2].try_into().ok()?);

                // 1.b.3 Check if the precedence byte is valid.
                if precedence_byte != 0x01 {
                    return None; // Invalid precedence byte.
                }

                // 1.b.4 Deserialize the precedence.
                let precedence = Some((
                    match u8::from_le_bytes(bytes[2..3].try_into().ok()?) {
                        0 => false,
                        1 => true,
                        _ => return None,
                    },
                    u64::from_le_bytes(bytes[3..11].try_into().ok()?),
                ));

                // 1.b.5 Construct the timed switch bool.
                let timed_switch_bool = TimedSwitchBool::new(default_value, precedence);

                // 1.b.6 Return the timed switch bool. Some(timed_switch_bool)
                Some(timed_switch_bool)
            }
            // 1.c Invalid length.
            _ => None,
        }
    }
}
