use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Contract id.
type ContractId = [u8; 32];

/// Redemption amount in satoshis.
type RedemptionAmountInSatoshis = u64;

/// A struct for containing ephemeral state differences to be applied for 'Graveyard'.
#[derive(Clone)]
pub struct GraveyardDelta {
    // Accounts to be buried and their corresponding redemption amounts owed to them.
    pub accounts_to_bury: HashMap<AccountKey, RedemptionAmountInSatoshis>,

    // Contracts to be buried.
    pub contracts_to_bury: Vec<ContractId>,

    // In-graveyard accounts to redeem their coins.
    pub redemptions: HashMap<AccountKey, RedemptionAmountInSatoshis>,
}

impl GraveyardDelta {
    /// Constructs a fresh new graveyard delta.
    pub fn fresh_new() -> Self {
        Self {
            accounts_to_bury: HashMap::new(),
            contracts_to_bury: Vec::new(),
            redemptions: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.accounts_to_bury.clear();
        self.contracts_to_bury.clear();
        self.redemptions.clear();
    }

    /// Checks if an account has just been ephemerally buried in the delta.
    pub fn is_account_ephemerally_buried(&self, account_key: AccountKey) -> bool {
        self.accounts_to_bury.contains_key(&account_key)
    }

    /// Checks if a contract has just been ephemerally buried in the delta.
    pub fn is_contract_ephemerally_buried(&self, contract_id: ContractId) -> bool {
        self.contracts_to_bury.contains(&contract_id)
    }

    /// Checks if an account has just been ephemerally redeemed in the delta.
    pub fn is_account_ephemerally_redeemed(&self, account_key: AccountKey) -> bool {
        self.redemptions.contains_key(&account_key)
    }

    /// Epheremally burries an account and the amount of satoshi they are owed for redemption.
    pub fn ephemerally_bury_account(&mut self, account_key: [u8; 32], redemption_amount: u64) {
        self.accounts_to_bury.insert(account_key, redemption_amount);
    }

    /// Ephemerally buries a contract.
    pub fn ephemerally_bury_contract(&mut self, contract_id: [u8; 32]) {
        self.contracts_to_bury.push(contract_id);
    }

    /// Epheremally redeems an account and its corresponding redemption amount.
    pub fn ephemerally_redeem_account_coins(
        &mut self,
        account_key: [u8; 32],
        redemption_amount: u64,
    ) {
        self.redemptions.insert(account_key, redemption_amount);
    }
}
