use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PrivilegesManagerAccountBody;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PrivilegesManagerContractBody;
use crate::inscriptive::privileges_manager::delta::delta::PrivilegesManagerDelta;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_long_val::timed_switch_long_val::TimedSwitchLongVal;
use crate::inscriptive::privileges_manager::errors::construction_error::PrivilegesManagerConstructionError;
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
const ACCOUNT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY: [u8; 1] = [0x05; 1];
const CONTRACT_LIVENESS_FLAG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];
const CONTRACT_IMMUTABILITY_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];
const CONTRACT_TAX_EXEMPTIONS_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];
const CONTRACT_STORAGE_LIMIT_SPECIAL_DB_KEY: [u8; 1] = [0x03; 1];
const CONTRACT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY: [u8; 1] = [0x04; 1];

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
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                PrivilegesManagerConstructionError::UnableToDeserializeAccountKeyBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // 4.2 Initialize the account elements.
            let mut account_liveness_flag: Option<LivenessFlag> = None;
            let mut account_hierarchy: Option<AccountHierarchy> = None;
            let mut account_txfee_exemptions: Option<Exemption> = None;
            let mut account_can_deploy_liquidity: Option<TimedSwitchBool> = None;
            let mut account_can_deploy_contract: Option<TimedSwitchBool> = None;
            let mut account_last_activity_timestamp: Option<u64> = None;

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

                    ACCOUNT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY => {
                        let account_last_activity_timestamp_deserialized = u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                            PrivilegesManagerConstructionError::UnableToDeserializeAccountLastActivityTimestampFromBytes(account_key, value.to_vec())
                                        })?);

                        account_last_activity_timestamp =
                            Some(account_last_activity_timestamp_deserialized);
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
                account_last_activity_timestamp.ok_or(
                    PrivilegesManagerConstructionError::AccountLastActivityTimestampNotPresent(
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
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                PrivilegesManagerConstructionError::UnableToDeserializeContractKeyBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // 5.2 Initialize the contract elements.
            let mut contract_liveness_flag: Option<LivenessFlag> = None;
            let mut contract_immutability: Option<bool> = None;
            let mut contract_tax_exemptions: Option<Exemption> = None;
            let mut contract_storage_limit: Option<TimedSwitchLongVal> = None;
            let mut contract_last_activity_timestamp: Option<u64> = None;

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
                    CONTRACT_STORAGE_LIMIT_SPECIAL_DB_KEY => {
                        let contract_storage_limit_deserialized = TimedSwitchLongVal::from_bytes(value.as_ref()).ok_or(
                            PrivilegesManagerConstructionError::UnableToDeserializeContractStorageLimitFromBytes(
                                contract_id,
                                value.to_vec(),
                            ),
                        )?;
                        contract_storage_limit = Some(contract_storage_limit_deserialized);
                    }
                    CONTRACT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY => {
                        let contract_last_activity_timestamp_deserialized = u64::from_le_bytes(
                            value.as_ref().try_into().map_err(|_| {
                                PrivilegesManagerConstructionError::UnableToDeserializeContractLastActivityTimestampFromBytes(
                                    contract_id,
                                    value.to_vec(),
                                )
                            })?,
                        );
                        contract_last_activity_timestamp =
                            Some(contract_last_activity_timestamp_deserialized);
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
                contract_storage_limit.ok_or(
                    PrivilegesManagerConstructionError::ContractStorageLimitNotPresent(contract_id),
                )?,
                contract_last_activity_timestamp.ok_or(
                    PrivilegesManagerConstructionError::ContractLastActivityTimestampNotPresent(
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
}
