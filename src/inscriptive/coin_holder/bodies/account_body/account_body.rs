use crate::transmutative::hash::{Hash, HashTag};

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for containing account balance and shadow allocs sum of all contracts.
#[derive(Clone)]
pub struct CHAccountBody {
    // Account balance.
    balance: SATOSHI_AMOUNT,

    // Individual shadow allocs sum of all contracts.
    shadow_allocs_sum: SATI_SATOSHI_AMOUNT,
}

impl CHAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(balance: SATOSHI_AMOUNT, shadow_allocs_sum: SATI_SATOSHI_AMOUNT) -> Self {
        Self {
            balance: balance,
            shadow_allocs_sum: shadow_allocs_sum,
        }
    }

    /// Returns the account balance.
    pub fn balance(&self) -> SATOSHI_AMOUNT {
        self.balance
    }

    /// Returns the account shadow allocs sum.
    pub fn shadow_allocs_sum(&self) -> SATI_SATOSHI_AMOUNT {
        self.shadow_allocs_sum
    }

    /// Updates the account balance.
    pub fn update_balance(&mut self, balance: SATOSHI_AMOUNT) {
        self.balance = balance;
    }

    /// Updates the account shadow allocs sum.
    pub fn update_shadow_allocs_sum(&mut self, shadow_allocs_sum: SATI_SATOSHI_AMOUNT) {
        self.shadow_allocs_sum = shadow_allocs_sum;
    }

    /// Hashes the account body with the `AccountBody` tag.
    pub fn tagged_hash(&self) -> [u8; 32] {
        let mut preimage: Vec<u8> = Vec::<u8>::new();

        preimage.extend(self.balance.to_le_bytes());
        preimage.extend(self.shadow_allocs_sum.to_le_bytes());

        preimage.hash(Some(HashTag::AccountBody))
    }
}
