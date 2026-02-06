use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Satoshi amount.
type SatoshiRedemptionAmount = u64;

/// A struct for containing epheremal state differences to be applied for 'Graveyard'.
#[derive(Clone)]
pub struct GraveyardDelta {
    // Accounts to destroy and the amount of satoshi they are owed for redemption.
    pub accounts_to_destroy: HashMap<AccountKey, SatoshiRedemptionAmount>,
}

impl GraveyardDelta {
    /// Constructs a fresh new graveyard delta.
    pub fn fresh_new() -> Self {
        Self {
            accounts_to_destroy: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.accounts_to_destroy.clear();
    }

    /// Epheremally inserts an account to destroy and the amount of satoshi they are owed for redemption.
    pub fn epheremally_insert_account_to_destroy(
        &mut self,
        account_key: [u8; 32],
        satoshi_redemption_amount: u64,
    ) -> bool {
        // 1 Check if the account is already in the delta.
        if self.accounts_to_destroy.contains_key(&account_key) {
            return false;
        }

        // 2 Insert the account to destroy and the amount of satoshi they are owed for redemption.
        self.accounts_to_destroy
            .insert(account_key, satoshi_redemption_amount);

        // 3 Return true.
        true
    }
}
