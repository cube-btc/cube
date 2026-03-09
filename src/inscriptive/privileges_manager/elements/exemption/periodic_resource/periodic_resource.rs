use serde::{Deserialize, Serialize};

/// Represents a periodically refilled resource, refilled in proportion to the time passed.
#[derive(Clone, Serialize, Deserialize)]
pub struct PeriodicResource {
    // The period in seconds.
    pub period: u64,

    // The maximum resource limit in that period.
    pub limit: u64,

    // The variable part of the resource.
    // This is the left amount of the resource from the latest consumption.
    pub latest_left: u64,

    // The timestamp of the latest consumption.
    pub latest_consumption_timestamp: u64,
}

impl PeriodicResource {
    /// Creates a new periodic resource with the given parameters.
    pub fn new(
        period: u64,
        limit: u64,
        latest_left: u64,
        latest_consumption_timestamp: u64,
    ) -> Self {
        Self {
            period,
            limit,
            latest_left,
            latest_consumption_timestamp,
        }
    }

    /// Updates the period.
    pub fn update_period(&mut self, new_period: u64) {
        self.period = new_period;
    }

    /// Updates the limit.
    pub fn update_limit(&mut self, new_limit: u64) {
        self.limit = new_limit;
    }

    /// Returns the current left by taking the refill into account since the latest consumption timestamp.
    pub fn current_left(&self, current_timestamp: u64) -> Option<u64> {
        // 1 Check if the current timestamp is before the latest consumption timestamp.
        if self.latest_consumption_timestamp > current_timestamp {
            return None;
        }

        // 2 Calculate the time passed since the latest consumption.
        let time_passed = current_timestamp - self.latest_consumption_timestamp;

        // 3 Check if the time passed is greater than or equal to the period.
        match time_passed >= self.period {
            // 3.a If the time passed is greater than the period.
            true => {
                // 3.a.1 The new amount is the upper bound itself (refilled to the fullest extent).
                let new_amount = self.limit;

                // 3.a.2 Return the new amount.
                return Some(new_amount);
            }

            // 3.b If the time passed is less than the period.
            false => {
                // 3.b.1 Calculate the refill amount proportional to the time passed.
                // The refill rate is limit / period per second.
                let refill_amount = (time_passed * self.limit) / self.period;

                // 3.b.2 Add the refill amount to the latest left.
                let new_amount = self.latest_left + refill_amount;

                // 3.b.3 Cap the new amount at the limit (don't exceed the maximum).
                let new_amount = new_amount.min(self.limit);

                // 3.b.4 Return the new amount.
                return Some(new_amount);
            }
        };
    }

    /// Refills the resource, then consumes the given amount, then updates and returns the new amount.
    pub fn refill_and_consume(
        &mut self,
        current_timestamp: u64,
        consume_amount: u64,
    ) -> Option<u64> {
        // 1 Return the current left.
        let current_left = self.current_left(current_timestamp)?;

        // 2 Check if there is enough resource to consume.
        if current_left < consume_amount {
            return None;
        }

        // 3 Calculate the new amount.
        let new_amount = current_left - consume_amount;

        // 4 Update the latest left.
        self.latest_left = new_amount;

        // 5 Update the latest consumption timestamp.
        self.latest_consumption_timestamp = current_timestamp;

        // 6 Return the new amount.
        Some(new_amount)
    }

    /// Serializes the periodic resource to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::with_capacity(32);

        // 2 Serialize the period.
        bytes.extend(self.period.to_le_bytes());

        // 3 Serialize the limit.
        bytes.extend(self.limit.to_le_bytes());

        // 4 Serialize the latest left.
        bytes.extend(self.latest_left.to_le_bytes());

        // 5 Serialize the latest consumption timestamp.
        bytes.extend(self.latest_consumption_timestamp.to_le_bytes());

        // 6 Return the bytes.
        bytes
    }

    /// Deserializes the periodic resource from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<PeriodicResource> {
        // 1 Check if the byte vector has the correct length.
        if bytes.len() != 32 {
            return None;
        }

        // 2 Deserialize the period.
        let period = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

        // 3 Deserialize the limit.
        let limit = u64::from_le_bytes(bytes[8..16].try_into().ok()?);

        // 4 Deserialize the latest left.
        let latest_left = u64::from_le_bytes(bytes[16..24].try_into().ok()?);

        // 5 Deserialize the latest consumption timestamp.
        let latest_consumption_timestamp = u64::from_le_bytes(bytes[24..32].try_into().ok()?);

        // 6 Construct the periodic resource.
        let periodic_resource = PeriodicResource {
            period,
            limit,
            latest_left,
            latest_consumption_timestamp,
        };

        // 6 Return the periodically refilled resource.
        Some(periodic_resource)
    }
}
