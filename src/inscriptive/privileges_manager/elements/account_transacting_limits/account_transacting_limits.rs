use crate::inscriptive::privileges_manager::elements::periodic_resource::periodic_resource::PeriodicResource;
use serde::{Deserialize, Serialize};

/// The account transacting limits (number of entries and ops per period).
#[derive(Clone, Serialize, Deserialize)]
pub struct AccountTransactingLimits {
    // TXCOUNT LIMIT.
    // The maximum number of transactions allowed in a given period.
    pub entrycount_limit: PeriodicResource,

    // OPSCOUNT LIMIT.
    // The maximum number of ops allowed to be executed in a given period.
    pub opscount_limit: PeriodicResource,
}

impl AccountTransactingLimits {
    /// Creates a new account transacting limits.
    pub fn new(entrycount_limit: PeriodicResource, opscount_limit: PeriodicResource) -> Self {
        Self {
            entrycount_limit,
            opscount_limit,
        }
    }

    /// Serializes the account transacting limits to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::with_capacity(48);

        // 2 Serialize the entrycount limit.
        bytes.extend(self.entrycount_limit.to_bytes());

        // 3 Serialize the opscount limit.
        bytes.extend(self.opscount_limit.to_bytes());

        // 4 Return the bytes.
        bytes
    }

    /// Deserializes the account transacting limits from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<AccountTransactingLimits> {
        // 1 Check if the byte vector has the correct length.
        if bytes.len() != 48 {
            return None;
        }

        // 2 Deserialize the entrycount limit.
        let entrycount_limit = PeriodicResource::from_bytes(bytes[0..24].try_into().ok()?)?;

        // 3 Deserialize the opscount limit.
        let opscount_limit = PeriodicResource::from_bytes(bytes[24..48].try_into().ok()?)?;

        // 4 Construct the account transacting limits.
        let account_transacting_limits = AccountTransactingLimits {
            entrycount_limit,
            opscount_limit,
        };

        // 5 Return the account transacting limits.
        Some(account_transacting_limits)
    }
}
