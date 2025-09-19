use std::collections::HashMap;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for containing state differences to be applied.
#[derive(Clone)]
pub struct CHAccountDelta {
    // New accounts to register.
    pub new_accounts_to_register: HashMap<ACCOUNT_KEY, SATOSHI_AMOUNT>,

    // Updated account balances for a given account.
    pub updated_account_balances: HashMap<ACCOUNT_KEY, SATOSHI_AMOUNT>,

    // Updated shadow allocs sums for a given account.
    pub updated_shadow_allocs_sums: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}

impl CHAccountDelta {
    /// Constructs a fresh new account delta.
    pub fn new() -> Self {
        Self {
            new_accounts_to_register: HashMap::new(),
            updated_account_balances: HashMap::new(),
            updated_shadow_allocs_sums: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_account_balances.clear();
        self.updated_shadow_allocs_sums.clear();
    }
}
