use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Redemption amount in satoshis.
type RedemptionAmountInSatoshis = u64;

/// A struct for containing epheremal state differences to be applied for 'Graveyard'.
#[derive(Clone)]
pub struct GraveyardDelta {
    // Accounts to be burried and their corresponding redemption amounts owed to them.
    pub accounts_to_burry: HashMap<AccountKey, RedemptionAmountInSatoshis>,

    // In-graveyard accounts to redeem their coins.
    pub redemptions: HashMap<AccountKey, RedemptionAmountInSatoshis>,
}

impl GraveyardDelta {
    /// Constructs a fresh new graveyard delta.
    pub fn fresh_new() -> Self {
        Self {
            accounts_to_burry: HashMap::new(),
            redemptions: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.accounts_to_burry.clear();
        self.redemptions.clear();
    }

    /// Checks if an account has just been epheremally burried in the delta.
    pub fn is_account_epheremally_burried(&self, account_key: AccountKey) -> bool {
        self.accounts_to_burry.contains_key(&account_key)
    }

    /// Checks if an account has just been epheremally redeemed in the delta.
    pub fn is_account_epheremally_redeemed(&self, account_key: AccountKey) -> bool {
        self.redemptions.contains_key(&account_key)
    }

    /// Epheremally burries an account and the amount of satoshi they are owed for redemption.
    pub fn epheremally_burry_account(&mut self, account_key: [u8; 32], redemption_amount: u64) {
        self.accounts_to_burry
            .insert(account_key, redemption_amount);
    }

    /// Epheremally redeems an account and its corresponding redemption amount.
    pub fn epheremally_redeem_account_coins(
        &mut self,
        account_key: [u8; 32],
        redemption_amount: u64,
    ) {
        self.redemptions.insert(account_key, redemption_amount);
    }
}
