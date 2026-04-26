use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PrivilegesManagerAccountBody;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PrivilegesManagerContractBody;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;
use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// A struct for containing epheremal state differences to be applied for `PrivilegesManager`.
#[derive(Clone)]
pub struct PrivilegesManagerDelta {
    pub new_accounts_to_register: HashMap<AccountKey, PrivilegesManagerAccountBody>,
    pub new_contracts_to_register: HashMap<ContractId, PrivilegesManagerContractBody>,
    pub updated_account_liveness_flags: HashMap<AccountKey, LivenessFlag>,
    pub updated_account_hierarchies: HashMap<AccountKey, AccountHierarchy>,
    pub updated_account_txfee_exemptions: HashMap<AccountKey, Exemption>,
    pub updated_account_can_deploy_liquidity: HashMap<AccountKey, TimedSwitchBool>,
    pub updated_account_can_deploy_contract: HashMap<AccountKey, TimedSwitchBool>,
    pub updated_contract_liveness_flags: HashMap<ContractId, LivenessFlag>,
    pub updated_contract_immutability_flags: HashMap<ContractId, bool>,
    pub updated_contract_tax_exemptions: HashMap<ContractId, Exemption>,
}

impl PrivilegesManagerDelta {
    /// Creates a fresh new delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: HashMap::new(),
            new_contracts_to_register: HashMap::new(),
            updated_account_liveness_flags: HashMap::new(),
            updated_account_hierarchies: HashMap::new(),
            updated_account_txfee_exemptions: HashMap::new(),
            updated_account_can_deploy_liquidity: HashMap::new(),
            updated_account_can_deploy_contract: HashMap::new(),
            updated_contract_liveness_flags: HashMap::new(),
            updated_contract_immutability_flags: HashMap::new(),
            updated_contract_tax_exemptions: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.new_contracts_to_register.clear();
        self.updated_account_liveness_flags.clear();
        self.updated_account_hierarchies.clear();
        self.updated_account_txfee_exemptions.clear();
        self.updated_account_can_deploy_liquidity.clear();
        self.updated_account_can_deploy_contract.clear();
        self.updated_contract_liveness_flags.clear();
        self.updated_contract_immutability_flags.clear();
        self.updated_contract_tax_exemptions.clear();
    }

    /// Checks if an account has just been epheremally registered in the delta.
    pub fn is_account_epheremally_registered(&self, account_key: AccountKey) -> bool {
        self.new_accounts_to_register.contains_key(&account_key)
    }

    /// Checks if a contract has just been epheremally registered in the delta.
    pub fn is_contract_epheremally_registered(&self, contract_id: ContractId) -> bool {
        self.new_contracts_to_register.contains_key(&contract_id)
    }

    /// Epheremally registers an account in the delta.
    pub fn epheremally_register_account(
        &mut self,
        account_key: AccountKey,
        account_body: PrivilegesManagerAccountBody,
    ) -> bool {
        if self.is_account_epheremally_registered(account_key) {
            return false;
        }

        self.new_accounts_to_register
            .insert(account_key, account_body);
        true
    }

    /// Epheremally registers a contract in the delta.
    pub fn epheremally_register_contract(
        &mut self,
        contract_id: ContractId,
        contract_body: PrivilegesManagerContractBody,
    ) -> bool {
        if self.is_contract_epheremally_registered(contract_id) {
            return false;
        }

        self.new_contracts_to_register
            .insert(contract_id, contract_body);
        true
    }
}