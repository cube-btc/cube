use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PrivilegesManagerAccountBody;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PrivilegesManagerContractBody;
use crate::inscriptive::privileges_manager::delta::delta::PrivilegesManagerDelta;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::account_transacting_limits::account_transacting_limits::AccountTransactingLimits;
use crate::inscriptive::privileges_manager::elements::account_txfee_privileges::account_txfee_privileges::AccountTxFeePrivileges;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::errors::construction_error::PrivilegesManagerConstructionError;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

const ACCOUNT_HIERARCHY_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];
const ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];
const ACCOUNT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];
const ACCOUNT_TXFEE_PRIVILEGES_SPECIAL_DB_KEY: [u8; 1] = [0x03; 1];
const ACCOUNT_TRANSACTING_LIMITS_SPECIAL_DB_KEY: [u8; 1] = [0x04; 1];
const ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY: [u8; 1] = [0x05; 1];
const ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY: [u8; 1] = [0x06; 1];

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
            let mut account_hierarchy: Option<AccountHierarchy> = None;
            let mut account_liveness_flag: Option<LivenessFlag> = None;
            let mut account_last_activity_timestamp: Option<u64> = None;
            let mut account_txfee_privileges: Option<AccountTxFeePrivileges> = None;
            let mut account_transacting_limits: Option<AccountTransactingLimits> = None;
            let mut account_can_deploy_liquidity: Option<bool> = None;
            let mut account_can_deploy_contract: Option<bool> = None;

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
                    // 4.4.3.a If the key is (0x00), it is a special key that corresponds to the account hierarchy element.
                    ACCOUNT_HIERARCHY_SPECIAL_DB_KEY => {
                        // 4.4.3.a.1 Get the account hierarchy bytecode.
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

                        // 4.4.3.a.2 Deserialize the account hierarchy from the bytecode.
                        let account_hierarchy_deserialized = AccountHierarchy::from_bytecode(account_hierarchy_bytecode).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountHierarchyFromBytecode(account_key, account_hierarchy_bytecode))?;

                        // 4.4.3.a.3 Update the account hierarchy.
                        account_hierarchy = Some(account_hierarchy_deserialized);
                    }
                    // 4.4.3.b If the key is (0x01), it is a special key that corresponds to the account liveness flag element.
                    ACCOUNT_LIVENESS_FLAG_SPECIAL_DB_KEY => {
                        // 4.4.3.b.1 Deserialize the accoubt liveness flag from bytes.
                        let account_liveness_flag_deserialized = LivenessFlag::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountLivenessFlagFromBytes(account_key, value.to_vec()))?;

                        // 4.4.3.b.2 Update the account liveness flag.
                        account_liveness_flag = Some(account_liveness_flag_deserialized);
                    }
                    // 4.4.3.c If the key is (0x02), it is a special key that corresponds to the account last activity timestamp element.
                    ACCOUNT_LAST_ACTIVITY_TIMESTAMP_SPECIAL_DB_KEY => {
                        // 4.4.3.c.1 Deserialize the account last activity timestamp from bytes.
                        let account_last_activity_timestamp_deserialized = u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                            PrivilegesManagerConstructionError::UnableToDeserializeAccountLastActivityTimestampFromBytes(account_key, value.to_vec())
                        })?);

                        // 4.4.3.c.2 Update the account last activity timestamp.
                        account_last_activity_timestamp =
                            Some(account_last_activity_timestamp_deserialized);
                    }
                    // 4.4.3.d If the key is (0x03), it is a special key that corresponds to the account tx fee privileges element.
                    ACCOUNT_TXFEE_PRIVILEGES_SPECIAL_DB_KEY => {
                        // 4.4.3.d.1 Deserialize the account tx fee privileges from bytes.
                        let account_txfee_privileges_deserialized = AccountTxFeePrivileges::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountTxFeePrivilegesFromBytes(account_key, value.to_vec()))?;

                        // 4.4.3.d.2 Update the account tx fee privileges.
                        account_txfee_privileges = Some(account_txfee_privileges_deserialized);
                    }
                    // 4.4.3.e If the key is (0x04), it is a special key that corresponds to the account transacting limits element.
                    ACCOUNT_TRANSACTING_LIMITS_SPECIAL_DB_KEY => {
                        // 4.4.3.e.1 Deserialize the account transacting limits from bytes.
                        let account_transacting_limits_deserialized = AccountTransactingLimits::from_bytes(value.as_ref()).ok_or(PrivilegesManagerConstructionError::UnableToDeserializeAccountTransactingLimitsFromBytes(account_key, value.to_vec()))?;

                        // 4.4.3.e.2 Update the account transacting limits.
                        account_transacting_limits = Some(account_transacting_limits_deserialized);
                    }
                    // 4.4.3.f If the key is (0x05), it is a special key that corresponds to the account can deploy liquidity element.
                    ACCOUNT_CAN_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY => {
                        // 4.4.3.f.1 Deserialize the account can deploy liquidity from bytes.
                        let account_can_deploy_liquidity_deserialized: bool = match value.len() {
                            0 => false,
                            1 => value.as_ref()[0] == 0x01,
                            _ => return Err(PrivilegesManagerConstructionError::InvalidAccountCanDeployLiquidityBytes(account_key, value.to_vec())),
                        };

                        // 4.4.3.f.2 Update the account can deploy liquidity.
                        account_can_deploy_liquidity =
                            Some(account_can_deploy_liquidity_deserialized);
                    }
                    // 4.4.3.g If the key is (0x06), it is a special key that corresponds to the account can deploy contract element.
                    ACCOUNT_CAN_DEPLOY_CONTRACT_SPECIAL_DB_KEY => {
                        // 4.4.3.g.1 Deserialize the account can deploy contract from bytes.
                        let account_can_deploy_contract_deserialized: bool = match value.len() {
                            0 => false,
                            1 => value.as_ref()[0] == 0x01,
                            _ => return Err(PrivilegesManagerConstructionError::InvalidAccountCanDeployContractBytes(account_key, value.to_vec())),
                        };

                        // 4.4.3.g.2 Update the account can deploy contract.
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
                account_hierarchy.ok_or(
                    PrivilegesManagerConstructionError::AccountHierarchyNotPresent(account_key),
                )?,
                account_liveness_flag.ok_or(
                    PrivilegesManagerConstructionError::AccountLivenessFlagNotPresent(account_key),
                )?,
                account_last_activity_timestamp.ok_or(
                    PrivilegesManagerConstructionError::AccountLastActivityTimestampNotPresent(
                        account_key,
                    ),
                )?,
                account_txfee_privileges.ok_or(
                    PrivilegesManagerConstructionError::AccountTxFeePrivilegesNotPresent(
                        account_key,
                    ),
                )?,
                account_transacting_limits.ok_or(
                    PrivilegesManagerConstructionError::AccountTransactingLimitsNotPresent(
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
