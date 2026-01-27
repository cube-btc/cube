use serde::{Deserialize, Serialize};

/// The account transacting limits (number of entries and ops per period).
#[derive(Clone, Serialize, Deserialize)]
pub struct AccountTransactingLimits {
    // TXCOUNT LIMIT.
    // The maximum number of transactions allowed in a given period.
    pub entrycount_limit: u64,
    pub entrycount_period: u64,

    // OPSCOUNT LIMIT.
    // The maximum number of ops allowed to be executed in a given period.
    pub opscount_limit: u64,
    pub opscount_period: u64,
}

impl AccountTransactingLimits {
    /// Creates a new account transacting limits.
    pub fn new(
        entrycount_limit: u64,
        entrycount_period: u64,
        opscount_limit: u64,
        opscount_period: u64,
    ) -> AccountTransactingLimits {
        Self {
            entrycount_limit,
            entrycount_period,
            opscount_limit,
            opscount_period,
        }
    }
}

impl AccountTransactingLimits {
    /// Serializes the account transacting limits to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::with_capacity(32);

        // 2 Serialize the entrycount limit.
        bytes.extend(self.entrycount_limit.to_le_bytes());

        // 3 Serialize the entrycount period.
        bytes.extend(self.entrycount_period.to_le_bytes());

        // 4 Serialize the opscount limit.
        bytes.extend(self.opscount_limit.to_le_bytes());

        // 5 Serialize the opscount period.
        bytes.extend(self.opscount_period.to_le_bytes());

        // 6 Return the bytes.
        bytes
    }

    /// Deserializes the account transacting limits from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<AccountTransactingLimits> {
        // 1 Check if the byte vector has the correct length.
        if bytes.len() != 32 {
            return None;
        }

        // 2 Deserialize the entrycount limit.
        let entrycount_limit = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

        // 3 Deserialize the entrycount period.
        let entrycount_period = u64::from_le_bytes(bytes[8..16].try_into().ok()?);

        // 4 Deserialize the opscount limit.
        let opscount_limit = u64::from_le_bytes(bytes[16..24].try_into().ok()?);

        // 5 Deserialize the opscount period.
        let opscount_period = u64::from_le_bytes(bytes[24..32].try_into().ok()?);

        // 6 Construct the account transacting limits.
        let account_transacting_limits = AccountTransactingLimits {
            entrycount_limit,
            entrycount_period,
            opscount_limit,
            opscount_period,
        };

        // 7 Return the account transacting limits.
        Some(account_transacting_limits)
    }
}
