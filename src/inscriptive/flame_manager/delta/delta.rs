use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// A struct for containing epheremal state differences to be applied for `FlameManager`.
#[derive(Clone)]
pub struct FMDelta {
    // New accounts to register.
    pub new_accounts_to_register: Vec<(AccountKey, Option<FMAccountFlameConfig>)>,

    // Updated flame configs for a given account.
    pub updated_flame_configs: HashMap<AccountKey, FMAccountFlameConfig>,
}

impl FMDelta {
    /// Constructs a fresh new flame manager delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: Vec::new(),
            updated_flame_configs: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_flame_configs.clear();
    }

    /// Checks if an account has just been epheremally registered in the delta.
    pub fn is_account_epheremally_registered(&self, account_key: AccountKey) -> bool {
        self.new_accounts_to_register
            .iter()
            .any(|(key, _)| key == &account_key)
    }

    /// Epheremally registers an account in the delta.
    pub fn epheremally_register_account(
        &mut self,
        account_key: AccountKey,
        flame_config: Option<FMAccountFlameConfig>,
    ) -> bool {
        // 1 Check if the account has just been epheremally registered in the delta.
        match self.is_account_epheremally_registered(account_key) {
            // 1.a The account has just been epheremally registered in the delta.
            true => return false,

            // 1.b The account has not just been epheremally registered in the delta.
            false => {
                // 1.b.1 Insert the account into the new accounts to register list in the delta.
                self.new_accounts_to_register
                    .push((account_key, flame_config));

                // 1.b.2 Return the result.
                true
            }
        }
    }

    /// Epheremally updates an account's flame config.
    pub fn epheremally_update_account_flame_config(
        &mut self,
        account_key: AccountKey,
        flame_config: FMAccountFlameConfig,
    ) {
        self.updated_flame_configs.insert(account_key, flame_config);
    }
}
