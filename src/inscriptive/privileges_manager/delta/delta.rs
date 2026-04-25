use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
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
        self.updated_account_liveness_flags.clear();
        self.updated_account_hierarchies.clear();
        self.updated_account_txfee_exemptions.clear();
        self.updated_account_can_deploy_liquidity.clear();
        self.updated_account_can_deploy_contract.clear();
        self.updated_contract_liveness_flags.clear();
        self.updated_contract_immutability_flags.clear();
        self.updated_contract_tax_exemptions.clear();
    }
}