use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PrivilegesManagerAccountBody;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PrivilegesManagerContractBody;
use crate::inscriptive::privileges_manager::delta::delta::PrivilegesManagerDelta;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;
use crate::inscriptive::privileges_manager::errors::construction_error::PrivilegesManagerConstructionError;
use crate::inscriptive::privileges_manager::errors::register_error::{
    PMRegisterAccountError, PMRegisterContractError,
};
use crate::inscriptive::privileges_manager::errors::update_error::{
    PMUpdateAccountError, PMUpdateContractError,
};
use crate::operative::run_args::chain::Chain;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

const ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];
const ACCOUNT_HIERARCHY_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];
const ACCOUNT_TXFEE_EXEMPTIONS_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];
const ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY: [u8; 1] = [0x03; 1];
const ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY: [u8; 1] = [0x04; 1];

const CONTRACT_LIVENESS_FLAG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];
const CONTRACT_IMMUTABILITY_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];
const CONTRACT_TAX_EXEMPTIONS_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];

/// A struct for managing the privileges and fee management resources of accounts and contracts.
#[allow(dead_code)]
pub struct PrivilegesManager {
    // In-memory accounts.
    in_memory_accounts: HashMap<AccountKey, PrivilegesManagerAccountBody>,

    // In-memory contracts.
    in_memory_contracts: HashMap<ContractId, PrivilegesManagerContractBody>,

    // On-disk accounts.
    on_disk_accounts: sled::Db,

    // On-disk contracts.
    on_disk_contracts: sled::Db,

    // State differences to be applied.
    delta: PrivilegesManagerDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: PrivilegesManagerDelta,
}

/// Guarded 'PrivilegesManager'.
#[allow(non_camel_case_types)]
pub type PRIVILEGES_MANAGER = Arc<Mutex<PrivilegesManager>>;

impl PrivilegesManager {
    /// Creates a new privileges manager.
    pub fn new(chain: Chain) -> Result<PRIVILEGES_MANAGER, PrivilegesManagerConstructionError> {
        // 1 Open the accounts db.
        let accounts_db_path = format!("storage/{}/privileges/accounts", chain.to_string());
        let accounts_db = sled::open(accounts_db_path)
            .map_err(PrivilegesManagerConstructionError::AccountsDBOpenError)?;

        // 2 Open the contracts db.
        let contracts_db_path = format!("storage/{}/privileges/contracts", chain.to_string());
        let contracts_db = sled::open(contracts_db_path)
            .map_err(PrivilegesManagerConstructionError::ContractsDBOpenError)?;

        // 3 Initialize the in-memory lists of account and contract bodies.
        let mut in_memory_accounts = HashMap::<AccountKey, PrivilegesManagerAccountBody>::new();
        let mut in_memory_contracts = HashMap::<ContractId, PrivilegesManagerContractBody>::new();

        // 4 Collect account bodies from the account database.
        for tree_name in accounts_db.tree_names() {
            // 4.1 Convert the tree name to a account key.
            let account_key: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(account_key) => account_key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 4.2 Initialize the account elements.
            let mut account_liveness_flag: Option<LivenessFlag> = None;
            let mut account_hierarchy: Option<AccountHierarchy> = None;
            let mut account_txfee_exemptions: Option<Exemption> = None;
            let mut account_can_deploy_liquidity: Option<TimedSwitchBool> = None;
            let mut account_can_deploy_contract: Option<TimedSwitchBool> = None;

            // 4.3 Open the tree associated with the account.
            let tree = accounts_db.open_tree(&tree_name).map_err(|e| {
                PrivilegesManagerConstructionError::AccountsTreeOpenError(account_key, e)
            })?;

            // 4.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 4.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(PrivilegesManagerConstructionError::AccountsTreeIterError(
                            account_key,
                            e,
                        ));
                    }
                };

                // 4.4.2 Convert the key to a byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    PrivilegesManagerConstructionError::UnableToDeserializeAccountKeyByteFromTreeKey(
                        account_key,
                        key.to_vec(),
                    )
                })?;

                // 4.4.3 Match the key byte.
                match key_byte {
                    ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY => {
                        let account_liveness_flag_deserialized = LivenessFlag::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountLivenessFlagFromBytes(account_key, value.to_vec()))?;

                        account_liveness_flag = Some(account_liveness_flag_deserialized);
                    }

                    ACCOUNT_HIERARCHY_SPECIAL_DB_KEY => {
                        let account_hierarchy_bytecode: u8 = match value.len() {
                            0 => return Err(
                                PrivilegesManagerConstructionError::InvalidAccountHierarchyBytecode(
                                    account_key,
                                    value.to_vec(),
                                ),
                            ),
                            1 => value.as_ref()[0],
                            _ => return Err(
                                PrivilegesManagerConstructionError::InvalidAccountHierarchyBytecode(
                                    account_key,
                                    value.to_vec(),
                                ),
                            ),
                        };

                        let account_hierarchy_deserialized = AccountHierarchy::from_bytecode(account_hierarchy_bytecode).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountHierarchyFromBytecode(account_key, account_hierarchy_bytecode))?;

                        account_hierarchy = Some(account_hierarchy_deserialized);
                    }

                    ACCOUNT_TXFEE_EXEMPTIONS_SPECIAL_DB_KEY => {
                        let account_txfee_exemptions_deserialized = Exemption::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountTxFeeExemptionsFromBytes(account_key, value.to_vec()))?;

                        account_txfee_exemptions = Some(account_txfee_exemptions_deserialized);
                    }

                    ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY => {
                        let account_can_deploy_liquidity_deserialized = TimedSwitchBool::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountCanDeployLiquidityFromBytes(account_key, value.to_vec()))?;

                        account_can_deploy_liquidity =
                            Some(account_can_deploy_liquidity_deserialized);
                    }

                    ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY => {
                        let account_can_deploy_contract_deserialized = TimedSwitchBool::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountCanDeployContractFromBytes(account_key, value.to_vec()))?;

                        account_can_deploy_contract =
                            Some(account_can_deploy_contract_deserialized);
                    }

                    _ => {
                        return Err(PrivilegesManagerConstructionError::InvalidAccountDbKeyByte(
                            account_key,
                            key.to_vec(),
                        ));
                    }
                }
            }

            // 4.5 Construct the account body.
            let account_body = PrivilegesManagerAccountBody::new(
                account_liveness_flag.ok_or(
                    PrivilegesManagerConstructionError::AccountLivenessFlagNotPresent(account_key),
                )?,
                account_hierarchy.ok_or(
                    PrivilegesManagerConstructionError::AccountHierarchyNotPresent(account_key),
                )?,
                account_txfee_exemptions.ok_or(
                    PrivilegesManagerConstructionError::AccountTxFeeExemptionsNotPresent(
                        account_key,
                    ),
                )?,
                account_can_deploy_liquidity.ok_or(
                    PrivilegesManagerConstructionError::AccountCanDeployLiquidityNotPresent(
                        account_key,
                    ),
                )?,
                account_can_deploy_contract.ok_or(
                    PrivilegesManagerConstructionError::AccountCanDeployContractNotPresent(
                        account_key,
                    ),
                )?,
            );

            // 4.6 Insert the account body into the in-memory list of accounts.
            in_memory_accounts.insert(account_key, account_body);
        }

        // 5 Collect contract bodies from the contract database.
        for tree_name in contracts_db.tree_names() {
            // 5.1 Convert the tree name to a contract id.
            let contract_id: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(contract_id) => contract_id,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 5.2 Initialize the contract elements.
            let mut contract_liveness_flag: Option<LivenessFlag> = None;
            let mut contract_immutability: Option<bool> = None;
            let mut contract_tax_exemptions: Option<Exemption> = None;

            // 5.3 Open the tree associated with the contract.
            let tree = contracts_db.open_tree(&tree_name).map_err(|e| {
                PrivilegesManagerConstructionError::ContractsTreeOpenError(contract_id, e)
            })?;

            // 5.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 5.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(PrivilegesManagerConstructionError::ContractsTreeIterError(
                            contract_id,
                            e,
                        ));
                    }
                };

                // 5.4.2 Convert the key to a byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    PrivilegesManagerConstructionError::UnableToDeserializeContractKeyByteFromTreeKey(
                        contract_id,
                        key.to_vec(),
                    )
                })?;

                // 5.4.3 Match the key byte.
                match key_byte {
                    CONTRACT_LIVENESS_FLAG_SPECIAL_DB_KEY => {
                        let contract_liveness_flag_deserialized = LivenessFlag::from_bytes(value.as_ref()).ok_or(
                            PrivilegesManagerConstructionError::UnableToDeserializeContractLivenessFlagFromBytes(
                                contract_id,
                                value.to_vec(),
                            ),
                        )?;
                        contract_liveness_flag = Some(contract_liveness_flag_deserialized);
                    }
                    CONTRACT_IMMUTABILITY_SPECIAL_DB_KEY => {
                        let immutability_bytecode: u8 = match value.len() {
                            0 => return Err(
                                PrivilegesManagerConstructionError::InvalidContractImmutabilityBytecode(
                                    contract_id,
                                    value.to_vec(),
                                ),
                            ),
                            1 => value.as_ref()[0],
                            _ => return Err(
                                PrivilegesManagerConstructionError::InvalidContractImmutabilityBytecode(
                                    contract_id,
                                    value.to_vec(),
                                ),
                            ),
                        };
                        contract_immutability = Some(match immutability_bytecode {
                            0 => false,
                            1 => true,
                            _ => return Err(
                                PrivilegesManagerConstructionError::InvalidContractImmutabilityBytecode(
                                    contract_id,
                                    value.to_vec(),
                                ),
                            ),
                        });
                    }
                    CONTRACT_TAX_EXEMPTIONS_SPECIAL_DB_KEY => {
                        let contract_tax_exemptions_deserialized = Exemption::from_bytes(value.as_ref()).ok_or(
                            PrivilegesManagerConstructionError::UnableToDeserializeContractTaxExemptionsFromBytes(
                                contract_id,
                                value.to_vec(),
                            ),
                        )?;
                        contract_tax_exemptions = Some(contract_tax_exemptions_deserialized);
                    }
                    _ => {
                        return Err(
                            PrivilegesManagerConstructionError::InvalidContractDbKeyByte(
                                contract_id,
                                key.to_vec(),
                            ),
                        );
                    }
                }
            }

            // 5.5 Construct the contract body.
            let contract_body = PrivilegesManagerContractBody::new(
                contract_liveness_flag.ok_or(
                    PrivilegesManagerConstructionError::ContractLivenessFlagNotPresent(contract_id),
                )?,
                contract_immutability.ok_or(
                    PrivilegesManagerConstructionError::ContractImmutabilityNotPresent(contract_id),
                )?,
                contract_tax_exemptions.ok_or(
                    PrivilegesManagerConstructionError::ContractTaxExemptionsNotPresent(
                        contract_id,
                    ),
                )?,
            );

            // 5.6 Insert the contract body into the in-memory list of contracts.
            in_memory_contracts.insert(contract_id, contract_body);
        }

        // 6 Construct the privileges manager.
        let privileges_manager = PrivilegesManager {
            in_memory_accounts,
            in_memory_contracts,
            on_disk_accounts: accounts_db,
            on_disk_contracts: contracts_db,
            delta: PrivilegesManagerDelta::fresh_new(),
            backup_of_delta: PrivilegesManagerDelta::fresh_new(),
        };

        // 7 Guard the privileges manager.
        let guarded_privileges_manager = Arc::new(Mutex::new(privileges_manager));

        // 8 Return the guarded privileges manager.
        Ok(guarded_privileges_manager)
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares privileges manager prior to each execution.
    pub fn pre_execution(&mut self) {
        self.backup_delta();
    }

    /// Returns the account body by account key.
    pub fn get_account_body_by_account_key(
        &self,
        account_key: AccountKey,
    ) -> Option<PrivilegesManagerAccountBody> {
        self.in_memory_accounts.get(&account_key).cloned()
    }

    /// Returns the contract body by contract id.
    pub fn get_contract_body_by_contract_id(
        &self,
        contract_id: ContractId,
    ) -> Option<PrivilegesManagerContractBody> {
        self.in_memory_contracts.get(&contract_id).cloned()
    }

    /// Returns whether account is permanently registered.
    pub fn is_account_registered(&self, account_key: AccountKey) -> bool {
        self.in_memory_accounts.contains_key(&account_key)
    }

    /// Returns whether contract is permanently registered.
    pub fn is_contract_registered(&self, contract_id: ContractId) -> bool {
        self.in_memory_contracts.contains_key(&contract_id)
    }

    /// Returns whether account has just been ephemerally registered.
    pub fn is_account_epheremally_registered(&self, account_key: AccountKey) -> bool {
        self.delta.is_account_epheremally_registered(account_key)
    }

    /// Returns whether contract has just been ephemerally registered.
    pub fn is_contract_epheremally_registered(&self, contract_id: ContractId) -> bool {
        self.delta.is_contract_epheremally_registered(contract_id)
    }

    /// Returns whether account is available in either permanent or ephemeral state.
    fn is_account_available(&self, account_key: AccountKey) -> bool {
        self.is_account_registered(account_key) || self.is_account_epheremally_registered(account_key)
    }

    /// Returns whether contract is available in either permanent or ephemeral state.
    fn is_contract_available(&self, contract_id: ContractId) -> bool {
        self.is_contract_registered(contract_id) || self.is_contract_epheremally_registered(contract_id)
    }

    /// Ephemerally registers a new account.
    pub fn register_account(
        &mut self,
        account_key: AccountKey,
        account_body: PrivilegesManagerAccountBody,
    ) -> Result<(), PMRegisterAccountError> {
        if self.is_account_epheremally_registered(account_key) {
            return Err(PMRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(
                account_key,
            ));
        }

        if self.is_account_registered(account_key) {
            return Err(PMRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(
                account_key,
            ));
        }

        self.delta
            .epheremally_register_account(account_key, account_body);
        Ok(())
    }

    /// Ephemerally registers a new contract.
    pub fn register_contract(
        &mut self,
        contract_id: ContractId,
        contract_body: PrivilegesManagerContractBody,
    ) -> Result<(), PMRegisterContractError> {
        if self.is_contract_epheremally_registered(contract_id) {
            return Err(PMRegisterContractError::ContractHasJustBeenEphemerallyRegistered(
                contract_id,
            ));
        }

        if self.is_contract_registered(contract_id) {
            return Err(PMRegisterContractError::ContractIsAlreadyPermanentlyRegistered(
                contract_id,
            ));
        }

        self.delta
            .epheremally_register_contract(contract_id, contract_body);
        Ok(())
    }

    /// Returns account liveness flag.
    pub fn get_account_liveness_flag(&self, account_key: AccountKey) -> Option<LivenessFlag> {
        if let Some(value) = self.delta.updated_account_liveness_flags.get(&account_key) {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_accounts_to_register.get(&account_key) {
            return Some(body.liveness_flag.clone());
        }

        self.in_memory_accounts
            .get(&account_key)
            .map(|body| body.liveness_flag.clone())
    }

    /// Returns account hierarchy.
    pub fn get_account_hierarchy(&self, account_key: AccountKey) -> Option<AccountHierarchy> {
        if let Some(value) = self.delta.updated_account_hierarchies.get(&account_key) {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_accounts_to_register.get(&account_key) {
            return Some(body.hierarchy.clone());
        }

        self.in_memory_accounts
            .get(&account_key)
            .map(|body| body.hierarchy.clone())
    }

    /// Returns account txfee exemptions.
    pub fn get_account_txfee_exemptions(&self, account_key: AccountKey) -> Option<Exemption> {
        if let Some(value) = self.delta.updated_account_txfee_exemptions.get(&account_key) {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_accounts_to_register.get(&account_key) {
            return Some(body.txfee_exemptions.clone());
        }

        self.in_memory_accounts
            .get(&account_key)
            .map(|body| body.txfee_exemptions.clone())
    }

    /// Returns account can-deploy-liquidity switch.
    pub fn get_account_can_deploy_liquidity(
        &self,
        account_key: AccountKey,
    ) -> Option<TimedSwitchBool> {
        if let Some(value) = self
            .delta
            .updated_account_can_deploy_liquidity
            .get(&account_key)
        {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_accounts_to_register.get(&account_key) {
            return Some(body.can_deploy_liquidity.clone());
        }

        self.in_memory_accounts
            .get(&account_key)
            .map(|body| body.can_deploy_liquidity.clone())
    }

    /// Returns account can-deploy-contract switch.
    pub fn get_account_can_deploy_contract(
        &self,
        account_key: AccountKey,
    ) -> Option<TimedSwitchBool> {
        if let Some(value) = self
            .delta
            .updated_account_can_deploy_contract
            .get(&account_key)
        {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_accounts_to_register.get(&account_key) {
            return Some(body.can_deploy_contract.clone());
        }

        self.in_memory_accounts
            .get(&account_key)
            .map(|body| body.can_deploy_contract.clone())
    }

    /// Returns contract liveness flag.
    pub fn get_contract_liveness_flag(&self, contract_id: ContractId) -> Option<LivenessFlag> {
        if let Some(value) = self.delta.updated_contract_liveness_flags.get(&contract_id) {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_contracts_to_register.get(&contract_id) {
            return Some(body.liveness_flag.clone());
        }

        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.liveness_flag.clone())
    }

    /// Returns contract immutability flag.
    pub fn get_contract_immutability(&self, contract_id: ContractId) -> Option<bool> {
        if let Some(value) = self
            .delta
            .updated_contract_immutability_flags
            .get(&contract_id)
        {
            return Some(*value);
        }

        if let Some(body) = self.delta.new_contracts_to_register.get(&contract_id) {
            return Some(body.immutability);
        }

        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.immutability)
    }

    /// Returns contract tax exemptions.
    pub fn get_contract_tax_exemptions(&self, contract_id: ContractId) -> Option<Exemption> {
        if let Some(value) = self.delta.updated_contract_tax_exemptions.get(&contract_id) {
            return Some(value.clone());
        }

        if let Some(body) = self.delta.new_contracts_to_register.get(&contract_id) {
            return Some(body.tax_exemptions.clone());
        }

        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.tax_exemptions.clone())
    }

    /// Epheremally updates account liveness flag.
    pub fn set_account_liveness_flag(
        &mut self,
        account_key: AccountKey,
        liveness_flag: LivenessFlag,
    ) -> Result<(), PMUpdateAccountError> {
        if !self.is_account_available(account_key) {
            return Err(PMUpdateAccountError::AccountIsNotRegistered(account_key));
        }

        self.delta
            .updated_account_liveness_flags
            .insert(account_key, liveness_flag);
        Ok(())
    }

    /// Epheremally updates account hierarchy.
    pub fn set_account_hierarchy(
        &mut self,
        account_key: AccountKey,
        hierarchy: AccountHierarchy,
    ) -> Result<(), PMUpdateAccountError> {
        if !self.is_account_available(account_key) {
            return Err(PMUpdateAccountError::AccountIsNotRegistered(account_key));
        }

        self.delta
            .updated_account_hierarchies
            .insert(account_key, hierarchy);
        Ok(())
    }

    /// Epheremally updates account txfee exemptions.
    pub fn set_account_txfee_exemptions(
        &mut self,
        account_key: AccountKey,
        txfee_exemptions: Exemption,
    ) -> Result<(), PMUpdateAccountError> {
        if !self.is_account_available(account_key) {
            return Err(PMUpdateAccountError::AccountIsNotRegistered(account_key));
        }

        self.delta
            .updated_account_txfee_exemptions
            .insert(account_key, txfee_exemptions);
        Ok(())
    }

    /// Epheremally updates account can-deploy-liquidity switch.
    pub fn set_account_can_deploy_liquidity(
        &mut self,
        account_key: AccountKey,
        can_deploy_liquidity: TimedSwitchBool,
    ) -> Result<(), PMUpdateAccountError> {
        if !self.is_account_available(account_key) {
            return Err(PMUpdateAccountError::AccountIsNotRegistered(account_key));
        }

        self.delta
            .updated_account_can_deploy_liquidity
            .insert(account_key, can_deploy_liquidity);
        Ok(())
    }

    /// Epheremally updates account can-deploy-contract switch.
    pub fn set_account_can_deploy_contract(
        &mut self,
        account_key: AccountKey,
        can_deploy_contract: TimedSwitchBool,
    ) -> Result<(), PMUpdateAccountError> {
        if !self.is_account_available(account_key) {
            return Err(PMUpdateAccountError::AccountIsNotRegistered(account_key));
        }

        self.delta
            .updated_account_can_deploy_contract
            .insert(account_key, can_deploy_contract);
        Ok(())
    }

    /// Epheremally updates contract liveness flag.
    pub fn set_contract_liveness_flag(
        &mut self,
        contract_id: ContractId,
        liveness_flag: LivenessFlag,
    ) -> Result<(), PMUpdateContractError> {
        if !self.is_contract_available(contract_id) {
            return Err(PMUpdateContractError::ContractIsNotRegistered(contract_id));
        }

        self.delta
            .updated_contract_liveness_flags
            .insert(contract_id, liveness_flag);
        Ok(())
    }

    /// Epheremally updates contract immutability.
    pub fn set_contract_immutability(
        &mut self,
        contract_id: ContractId,
        immutability: bool,
    ) -> Result<(), PMUpdateContractError> {
        if !self.is_contract_available(contract_id) {
            return Err(PMUpdateContractError::ContractIsNotRegistered(contract_id));
        }

        self.delta
            .updated_contract_immutability_flags
            .insert(contract_id, immutability);
        Ok(())
    }

    /// Epheremally updates contract tax exemptions.
    pub fn set_contract_tax_exemptions(
        &mut self,
        contract_id: ContractId,
        tax_exemptions: Exemption,
    ) -> Result<(), PMUpdateContractError> {
        if !self.is_contract_available(contract_id) {
            return Err(PMUpdateContractError::ContractIsNotRegistered(contract_id));
        }

        self.delta
            .updated_contract_tax_exemptions
            .insert(contract_id, tax_exemptions);
        Ok(())
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        self.restore_delta();
    }

    /// Applies all epheremal changes from delta into permanent in-memory and on-disk state.
    pub fn apply_changes(&mut self) -> Result<(), sled::Error> {
        // 1 Register new accounts.
        for (account_key, account_body) in self.delta.new_accounts_to_register.iter() {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(
                    ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY,
                    account_body.liveness_flag.to_bytes(),
                )?;
                tree.insert(
                    ACCOUNT_HIERARCHY_SPECIAL_DB_KEY,
                    [account_body.hierarchy.to_bytecode()].as_slice(),
                )?;
                tree.insert(
                    ACCOUNT_TXFEE_EXEMPTIONS_SPECIAL_DB_KEY,
                    account_body.txfee_exemptions.to_bytes(),
                )?;
                tree.insert(
                    ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY,
                    account_body.can_deploy_liquidity.to_bytes(),
                )?;
                tree.insert(
                    ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY,
                    account_body.can_deploy_contract.to_bytes(),
                )?;
            }

            self.in_memory_accounts
                .insert(*account_key, account_body.clone());
        }

        // 2 Register new contracts.
        for (contract_id, contract_body) in self.delta.new_contracts_to_register.iter() {
            {
                let tree = self.on_disk_contracts.open_tree(contract_id)?;
                tree.insert(
                    CONTRACT_LIVENESS_FLAG_SPECIAL_DB_KEY,
                    contract_body.liveness_flag.to_bytes(),
                )?;
                let immutability_byte = if contract_body.immutability { 1u8 } else { 0u8 };
                tree.insert(
                    CONTRACT_IMMUTABILITY_SPECIAL_DB_KEY,
                    [immutability_byte].as_slice(),
                )?;
                tree.insert(
                    CONTRACT_TAX_EXEMPTIONS_SPECIAL_DB_KEY,
                    contract_body.tax_exemptions.to_bytes(),
                )?;
            }

            self.in_memory_contracts
                .insert(*contract_id, contract_body.clone());
        }

        // 3 Save updated account liveness flags.
        for (account_key, liveness_flag) in self.delta.updated_account_liveness_flags.iter() {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY, liveness_flag.to_bytes())?;
            }

            if let Some(account_body) = self.in_memory_accounts.get_mut(account_key) {
                account_body.liveness_flag = liveness_flag.clone();
            }
        }

        // 4 Save updated account hierarchies.
        for (account_key, hierarchy) in self.delta.updated_account_hierarchies.iter() {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(ACCOUNT_HIERARCHY_SPECIAL_DB_KEY, [hierarchy.to_bytecode()].as_slice())?;
            }

            if let Some(account_body) = self.in_memory_accounts.get_mut(account_key) {
                account_body.hierarchy = hierarchy.clone();
            }
        }

        // 5 Save updated account txfee exemptions.
        for (account_key, txfee_exemptions) in self.delta.updated_account_txfee_exemptions.iter() {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(
                    ACCOUNT_TXFEE_EXEMPTIONS_SPECIAL_DB_KEY,
                    txfee_exemptions.to_bytes(),
                )?;
            }

            if let Some(account_body) = self.in_memory_accounts.get_mut(account_key) {
                account_body.txfee_exemptions = txfee_exemptions.clone();
            }
        }

        // 6 Save updated account can-deploy-liquidity switches.
        for (account_key, can_deploy_liquidity) in
            self.delta.updated_account_can_deploy_liquidity.iter()
        {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(
                    ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY,
                    can_deploy_liquidity.to_bytes(),
                )?;
            }

            if let Some(account_body) = self.in_memory_accounts.get_mut(account_key) {
                account_body.can_deploy_liquidity = can_deploy_liquidity.clone();
            }
        }

        // 7 Save updated account can-deploy-contract switches.
        for (account_key, can_deploy_contract) in self.delta.updated_account_can_deploy_contract.iter()
        {
            {
                let tree = self.on_disk_accounts.open_tree(account_key)?;
                tree.insert(
                    ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY,
                    can_deploy_contract.to_bytes(),
                )?;
            }

            if let Some(account_body) = self.in_memory_accounts.get_mut(account_key) {
                account_body.can_deploy_contract = can_deploy_contract.clone();
            }
        }

        // 8 Save updated contract liveness flags.
        for (contract_id, liveness_flag) in self.delta.updated_contract_liveness_flags.iter() {
            {
                let tree = self.on_disk_contracts.open_tree(contract_id)?;
                tree.insert(CONTRACT_LIVENESS_FLAG_SPECIAL_DB_KEY, liveness_flag.to_bytes())?;
            }

            if let Some(contract_body) = self.in_memory_contracts.get_mut(contract_id) {
                contract_body.liveness_flag = liveness_flag.clone();
            }
        }

        // 9 Save updated contract immutability flags.
        for (contract_id, immutability) in self.delta.updated_contract_immutability_flags.iter() {
            {
                let tree = self.on_disk_contracts.open_tree(contract_id)?;
                let immutability_byte = if *immutability { 1u8 } else { 0u8 };
                tree.insert(CONTRACT_IMMUTABILITY_SPECIAL_DB_KEY, [immutability_byte].as_slice())?;
            }

            if let Some(contract_body) = self.in_memory_contracts.get_mut(contract_id) {
                contract_body.immutability = *immutability;
            }
        }

        // 10 Save updated contract tax exemptions.
        for (contract_id, tax_exemptions) in self.delta.updated_contract_tax_exemptions.iter() {
            {
                let tree = self.on_disk_contracts.open_tree(contract_id)?;
                tree.insert(CONTRACT_TAX_EXEMPTIONS_SPECIAL_DB_KEY, tax_exemptions.to_bytes())?;
            }

            if let Some(contract_body) = self.in_memory_contracts.get_mut(contract_id) {
                contract_body.tax_exemptions = tax_exemptions.clone();
            }
        }

        Ok(())
    }

    /// Clears all epheremal changes from delta and backup.
    pub fn flush_delta(&mut self) {
        self.delta.flush();
        self.backup_of_delta.flush();
    }
}

/// Erases the privileges manager by db paths.
pub fn erase_privileges_manager(chain: Chain) {
    let accounts_db_path = format!("storage/{}/privileges/accounts", chain.to_string());
    let _ = std::fs::remove_dir_all(accounts_db_path);

    let contracts_db_path = format!("storage/{}/privileges/contracts", chain.to_string());
    let _ = std::fs::remove_dir_all(contracts_db_path);
}
