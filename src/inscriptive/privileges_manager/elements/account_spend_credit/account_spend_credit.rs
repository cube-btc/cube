use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountSpendCredit {
    // Secondary, direct spending credit.
    pub direct_spending_credit: u64,

    // Primary, periodic spending credit.
    pub periodic_spending_credit: u64,
    pub periodic_spending_credit_period: u64,
    pub periodic_spending_credit_left: u64,
}

impl AccountSpendCredit {
    // Constructs a fresh new account spend credit.
    pub fn new(
        // Secondary, direct spending credit.
        direct_spending_credit: u64,

        // Primary, periodic spending credit.
        periodic_spending_credit: u64,
        periodic_spending_credit_period: u64,
        periodic_spending_credit_left: u64,
    ) -> Self {
        Self {
            direct_spending_credit,
            periodic_spending_credit,
            periodic_spending_credit_period,
            periodic_spending_credit_left,
        }
    }

    /// Serializes the account spend credit to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create a new vector of bytes with the capacity for the 4 fields.
        let mut bytes = Vec::<u8>::with_capacity(32); // 8 bytes for each of the 4 fields.

        // 2 Serialize the direct spending credit.
        bytes.extend(self.direct_spending_credit.to_le_bytes());

        // 3 Serialize the periodic spending credit.
        bytes.extend(self.periodic_spending_credit.to_le_bytes());

        // 4 Serialize the periodic spending credit period.
        bytes.extend(self.periodic_spending_credit_period.to_le_bytes());

        // 5 Serialize the periodic spending credit left.
        bytes.extend(self.periodic_spending_credit_left.to_le_bytes());

        // 6 Return the bytes.
        bytes
    }

    /// Deserializes the account spend credit from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<AccountSpendCredit> {
        // 1 Check if there are enough bytes for the account spend credit.
        if bytes.len() != 32 {
            return None;
        }

        // 2 Deserialize the direct spending credit.
        let direct_spending_credit = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

        // 3 Deserialize the periodic spending credit.
        let periodic_spending_credit = u64::from_le_bytes(bytes[8..16].try_into().ok()?);

        // 4 Deserialize the periodic spending credit period.
        let periodic_spending_credit_period = u64::from_le_bytes(bytes[16..24].try_into().ok()?);

        // 5 Deserialize the periodic spending credit left.
        let periodic_spending_credit_left = u64::from_le_bytes(bytes[24..32].try_into().ok()?);

        // 6 Construct the account spend credit.
        let account_spend_credit = Self {
            direct_spending_credit,
            periodic_spending_credit,
            periodic_spending_credit_period,
            periodic_spending_credit_left,
        };

        // 7 Return the account spend credit.
        Some(account_spend_credit)
    }
}
