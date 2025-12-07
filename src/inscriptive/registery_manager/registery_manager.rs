use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::executive::executable::compiler::compiler::ExecutableCompiler;
use crate::executive::executable::executable::Executable;
use crate::inscriptive::registery_manager::bodies::account_body::account_body::RMAccountBody;
use crate::inscriptive::registery_manager::bodies::account_body::flame_config::flame_config::FlameConfig;
use crate::inscriptive::registery_manager::bodies::contract_body::contract_body::RMContractBody;
use crate::inscriptive::registery_manager::delta::delta::RMDelta;
use crate::inscriptive::registery_manager::errors::apply_changes_error::RMApplyChangesError;
use crate::inscriptive::registery_manager::errors::construction_error::RMConstructionError;
use crate::inscriptive::registery_manager::errors::increment_account_call_counter_error::RMIncrementAccountCallCounterError;
use crate::inscriptive::registery_manager::errors::increment_contract_call_counter_error::RMIncrementContractCallCounterError;
use crate::inscriptive::registery_manager::errors::reconfig_account_error::RMReconfigAccountError;
use crate::inscriptive::registery_manager::errors::register_account_error::RMRegisterAccountError;
use crate::inscriptive::registery_manager::errors::register_contract_error::RMRegisterContractError;
use crate::operative::Chain;
use secp::Point;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account Key.
type AccountKey = [u8; 32];

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type AccountSecondaryAggregationKey = Vec<u8>;

/// Contract ID.
type ContractId = [u8; 32];

/// Rank of an account or contract.
type Rank = u32;

/// Special db key for the registery index (0x00..).
const REGISTERY_INDEX_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];

/// Special db key for the call counter (0x01..).
const CALL_COUNTER_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];

/// Special db key for the program (0x02..).
const PROGRAM_BYTES_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];

/// Special db key for the primary BLS key (0x03..).
const BLS_KEY_SPECIAL_DB_KEY: [u8; 1] = [0x03; 1];

/// Special db key for the secondary aggregation key (0x04..).
const SECONDARY_AGGREGATION_KEY_SPECIAL_DB_KEY: [u8; 1] = [0x04; 1];

/// Special db key for the flame config (0x05..).
const FLAME_CONFIG_SPECIAL_DB_KEY: [u8; 1] = [0x05; 1];

/// A struct for managing the registery of accounts and contracts.
#[allow(dead_code)]
pub struct RegisteryManager {
    // In-memory list of account & contract bodies.
    in_memory_accounts: HashMap<AccountKey, RMAccountBody>,
    in_memory_contracts: HashMap<ContractId, RMContractBody>,

    // In-memory list of account & contract ranks for fast access.
    in_memory_account_ranks: HashMap<Rank, AccountKey>,
    in_memory_contract_ranks: HashMap<Rank, ContractId>,

    // On-disk dbs for storing the account & contract bodies and ranks.
    on_disk_accounts: sled::Db,
    on_disk_contracts: sled::Db,

    // State differences to be applied.
    delta: RMDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: RMDelta,
}

/// Guarded 'RegisteryManager'.
#[allow(non_camel_case_types)]
pub type REGISTERY_MANAGER = Arc<Mutex<RegisteryManager>>;

impl RegisteryManager {
    /// Constructs a fresh new registery manager.
    pub fn new(chain: Chain) -> Result<REGISTERY_MANAGER, RMConstructionError> {
        // 1 Open the accounts db.
        let accounts_db_path = format!("storage/{}/registery/accounts", chain.to_string());
        let accounts_db =
            sled::open(accounts_db_path).map_err(RMConstructionError::AccountsDBOpenError)?;

        // 2 Open the contracts db.
        let contracts_db_path = format!("storage/{}/registery/contracts", chain.to_string());
        let contracts_db =
            sled::open(contracts_db_path).map_err(RMConstructionError::ContractsDBOpenError)?;

        // 3 Initialize the in-memory lists of account & contract bodies.
        let mut in_memory_accounts = HashMap::<AccountKey, RMAccountBody>::new();
        let mut in_memory_contracts = HashMap::<ContractId, RMContractBody>::new();

        // 4 Iterate over all items in the accounts db to collect the account bodies.
        for tree_name in accounts_db.tree_names() {
            // 4.1 Convert the tree name to a account key.
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                RMConstructionError::UnableToDeserializeAccountKeyBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // 4.2 Initialize the registery index and call counter to zero.
            let mut registery_index = 0;

            // 4.3 Initialize the call counter to zero.
            let mut call_counter = 0;

            // 4.3 Initialize the BLS key to an empty byte array.
            let mut bls_key: Option<AccountBLSKey> = None;

            // 4.4 Initialize the secondary aggregation key to None.
            let mut secondary_aggregation_key: Option<Vec<u8>> = None;

            // 4.5 Initialize the flame config to a fresh new flame config.
            let mut flame_config: Option<FlameConfig> = None;

            // 4.3 Open the tree associated with the account.
            let tree = accounts_db
                .open_tree(&tree_name)
                .map_err(|e| RMConstructionError::AccountsTreeOpenError(account_key, e))?;

            // 4.4 Iterate over all items in the tree.
            // NOTE: There should be only two iterations in the tree, one for the registery index and one for the call counter.
            for item in tree.iter() {
                // 4.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(RMConstructionError::AccountsTreeIterError(account_key, e));
                    }
                };

                // 4.4.2 Convert the tree key to the single db key byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    RMConstructionError::UnableToDeserializeAccountDbKeyByteFromTreeKey(
                        account_key,
                        key.to_vec(),
                    )
                })?;

                // 4.4.3 Match the db key byte.
                match key_byte {
                    // 0x00 key byte represents the registery index.
                    REGISTERY_INDEX_SPECIAL_DB_KEY => {
                        // Convert the value to a registery index bytes.
                        let registery_index_bytes: [u8; 4] = value.as_ref().try_into().map_err(|_| {
                        RMConstructionError::UnableToDeserializeAccountRegisteryIndexBytesFromTreeValue(account_key, value.to_vec())
                    })?;

                        // Update the registery index.
                        registery_index = u32::from_le_bytes(registery_index_bytes);
                    }
                    // 0x01 key byte represents the call counter.
                    CALL_COUNTER_SPECIAL_DB_KEY => {
                        // Convert the value to a call counter bytes.
                        let call_counter_bytes: [u8; 8] = value.as_ref().try_into().map_err(|_| {
                        RMConstructionError::UnableToDeserializeAccountCallCounterBytesFromTreeValue(account_key, value.to_vec())
                    })?;

                        // Update the call counter.
                        call_counter = u64::from_le_bytes(call_counter_bytes);
                    }
                    // 0x03 key byte represents the primary BLS key.
                    BLS_KEY_SPECIAL_DB_KEY => {
                        if value.as_ref().len() > 0 {
                            // Get the primary BLS key bytes.
                            let bls_key_bytes: [u8; 48] = value.as_ref().try_into().map_err(|_| {
                                RMConstructionError::UnableToDeserializeAccountPrimaryBLSKeyBytesFromTreeValue(account_key, value.to_vec())
                            })?;

                            // Update the primary BLS key.
                            bls_key = Some(bls_key_bytes);
                        }
                    }
                    // 0x04 key byte represents the secondary aggregation key.
                    SECONDARY_AGGREGATION_KEY_SPECIAL_DB_KEY => {
                        if value.as_ref().len() > 0 {
                            // Convert the value to a secondary aggregation key bytes.
                            let secondary_aggregation_key_bytes: Vec<u8> = value.as_ref().to_vec();

                            // If the secondary aggregation key bytes are not empty, update the secondary aggregation key.
                            if secondary_aggregation_key_bytes.len() > 0 {
                                secondary_aggregation_key = Some(secondary_aggregation_key_bytes);
                            }
                        }
                    }
                    // 0x05 key byte represents the flame config.
                    FLAME_CONFIG_SPECIAL_DB_KEY => {
                        if value.as_ref().len() > 0 {
                            // Deserialize the flame config from bytes.
                            let flame_config_deserialized = FlameConfig::from_db_value_bytes(value.as_ref()).ok_or(RMConstructionError::UnableToDeserializeAccountFlameConfigBytesFromTreeValue(account_key, value.to_vec()))?;

                            // Update the flame config.
                            flame_config = Some(flame_config_deserialized);
                        }
                    }
                    // Invalid db key byte.
                    _ => {
                        return Err(RMConstructionError::InvalidAccountDbKeyByte(
                            account_key,
                            key.to_vec(),
                        ));
                    }
                }
            }

            // 4.5 Construct the account body with the collected registery index and call counter values.
            let account_body = RMAccountBody::new(
                registery_index,
                call_counter,
                bls_key,
                secondary_aggregation_key,
                flame_config,
            );

            // 4.6 Insert the account body into the in-memory list of accounts.
            in_memory_accounts.insert(account_key, account_body);
        }

        // 5 Iterate over all items in the contracts db to collect the contract bodies.
        for tree_name in contracts_db.tree_names() {
            // 5.1 Convert the tree name to a contract id.
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                RMConstructionError::UnableToDeserializeContractKeyBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // 5.2 Initialize the registery index and call counter to zero.
            let mut registery_index = 0;

            // 5.3 Initialize the call counter to zero.
            let mut call_counter = 0;

            // 5.4 Construct a placeholder executable.
            let mut executable = Executable::placeholder_executable();

            // 5.5 Open the tree associated with the contract.
            let tree = contracts_db
                .open_tree(&tree_name)
                .map_err(|e| RMConstructionError::ContractsTreeOpenError(contract_id, e))?;

            // 5.6 Iterate over all items in the tree.
            // NOTE: There should be only two iterations in the tree, one for the registery index and one for the call counter.
            for item in tree.iter() {
                // 5.6.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(RMConstructionError::ContractsTreeIterError(contract_id, e));
                    }
                };

                // 5.6.2 Convert the tree key to the single db key byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    RMConstructionError::UnableToDeserializeContractDbKeyByteFromTreeKey(
                        contract_id,
                        key.to_vec(),
                    )
                })?;

                // 5.6.3 Match the db key byte.
                match key_byte {
                    // 0x00 key byte represents the registery index.
                    REGISTERY_INDEX_SPECIAL_DB_KEY => {
                        // Convert the value to a registery index bytes.
                        let registery_index_bytes: [u8; 4] = value.as_ref().try_into().map_err(|_| {
                            RMConstructionError::UnableToDeserializeContractRegisteryIndexBytesFromTreeValue(contract_id, value.to_vec())
                        })?;

                        // Update the registery index.
                        registery_index = u32::from_le_bytes(registery_index_bytes);
                    }
                    // 0x01 key byte represents the call counter.
                    CALL_COUNTER_SPECIAL_DB_KEY => {
                        // Convert the value to a call counter bytes.
                        let call_counter_bytes: [u8; 8] = value.as_ref().try_into().map_err(|_| {
                            RMConstructionError::UnableToDeserializeContractCallCounterBytesFromTreeValue(contract_id, value.to_vec())
                        })?;

                        // Update the call counter.
                        call_counter = u64::from_le_bytes(call_counter_bytes);
                    }
                    // 0x02 key byte represents the program.
                    PROGRAM_BYTES_SPECIAL_DB_KEY => {
                        // Convert the value to a executable bytes.
                        let program_bytes: Vec<u8> = value.as_ref().to_vec();

                        // Decompile the executable from bytecode and update the executable.
                        executable = Executable::decompile(&mut program_bytes.into_iter())
                            .map_err(|e| {
                                RMConstructionError::ContractExecutableDecompileError(
                                    contract_id,
                                    e,
                                )
                            })?;
                    }
                    // Invalid db key byte.
                    _ => {
                        return Err(RMConstructionError::InvalidContractDbKeyByte(
                            contract_id,
                            key.to_vec(),
                        ));
                    }
                }
            }

            // 5.7 Construct the contract body with the collected registery index and call counter values.
            let contract_body = RMContractBody::new(registery_index, call_counter, executable);

            // 5.8 Insert the contract body into the in-memory list of contracts.
            in_memory_contracts.insert(contract_id, contract_body);
        }

        // 7 Rank accounts.
        let in_memory_account_ranks = Self::rank_accounts(&in_memory_accounts);

        // 8 Rank contracts.
        let in_memory_contract_ranks = Self::rank_contracts(&in_memory_contracts);

        // 9 Construct the registery manager.
        let registery_manager = RegisteryManager {
            in_memory_accounts,
            in_memory_contracts,
            in_memory_account_ranks,
            in_memory_contract_ranks,
            on_disk_accounts: accounts_db,
            on_disk_contracts: contracts_db,
            delta: RMDelta::fresh_new(),
            backup_of_delta: RMDelta::fresh_new(),
        };

        // 10 Guard the registery manager.
        let guarded_registery_manager = Arc::new(Mutex::new(registery_manager));

        // 11 Return the guarded registery manager.
        Ok(guarded_registery_manager)
    }

    /// Ranks accounts by call counter (descending) and registery index (ascending as tiebreaker).
    /// Returns a HashMap where keys are ranks starting from 1.
    fn rank_accounts(accounts: &HashMap<AccountKey, RMAccountBody>) -> HashMap<Rank, AccountKey> {
        // 1 Collect the ranking triples (account key, registery index, call counter).
        let mut ranking_triples: Vec<(AccountKey, u32, u64)> = accounts
            .iter()
            .map(|(account_key, account_body)| {
                (
                    account_key.to_owned(),
                    account_body.registery_index,
                    account_body.call_counter,
                )
            })
            .collect();

        // 2 Sort the ranking triples by call counter (descending), then by registery index (ascending) as tiebreaker.
        ranking_triples.sort_by(
            |(_, registery_index_a, call_counter_a), (_, registery_index_b, call_counter_b)| {
                // 2.1 Primary sort: call counter (descending).
                call_counter_b
                    .cmp(call_counter_a)
                    // 2.2 Secondary sort: registery index (ascending) as tiebreaker.
                    .then(registery_index_a.cmp(registery_index_b))
            },
        );

        // 3 Initialize the ranked accounts list.
        let mut ranked_accounts = HashMap::<Rank, AccountKey>::new();

        // 4 Calculate the ranks and insert the account keys and ranks into the ranked list.
        for (index, (account_key, _, _)) in ranking_triples.into_iter().enumerate() {
            // 4.1 Calculate the rank.
            // NOTE: Ranking count starts from 1.
            let rank = (index + 1) as Rank;

            // 4.2 Insert the account key and rank into the ranked list.
            ranked_accounts.insert(rank, account_key.to_owned());
        }

        // 5 Return the ranked accounts list.
        ranked_accounts
    }

    /// Ranks contracts by call counter (descending) and registery index (ascending as tiebreaker).
    /// Returns a HashMap where keys are ranks starting from 1.
    fn rank_contracts(
        contracts: &HashMap<ContractId, RMContractBody>,
    ) -> HashMap<Rank, ContractId> {
        // 1 Collect the ranking triples (contract id, registery index, call counter).
        let mut ranking_triples: Vec<(ContractId, u32, u64)> = contracts
            .iter()
            .map(|(contract_id, contract_body)| {
                (
                    contract_id.to_owned(),
                    contract_body.registery_index,
                    contract_body.call_counter,
                )
            })
            .collect();

        // 2 Sort the ranking triples by call counter (descending), then by registery index (ascending) as tiebreaker.
        ranking_triples.sort_by(
            |(_, registery_index_a, call_counter_a), (_, registery_index_b, call_counter_b)| {
                // 2.1 Primary sort: call counter (descending).
                call_counter_b
                    .cmp(call_counter_a)
                    // 2.2 Secondary sort: registery index (ascending) as tiebreaker.
                    .then(registery_index_a.cmp(registery_index_b))
            },
        );

        // 3 Initialize the ranked contracts list.
        let mut ranked_contracts = HashMap::<Rank, ContractId>::new();

        // 4 Calculate the ranks and insert the contract ids and ranks into the ranked list.
        for (index, (contract_id, _, _)) in ranking_triples.into_iter().enumerate() {
            // 4.1 Calculate the rank.
            // NOTE: Ranking count starts from 1.
            let rank = (index + 1) as Rank;

            // 4.2 Insert the contract id and rank into the ranked list.
            ranked_contracts.insert(rank, contract_id.to_owned());
        }

        // 5 Return the ranked contracts list.
        ranked_contracts
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares the registery manager prior to each execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn pre_execution(&mut self) {
        self.backup_delta();
    }

    /// Checks if an account is permanently registered.
    pub fn is_account_registered(&self, account_key: AccountKey) -> bool {
        self.in_memory_accounts.contains_key(&account_key)
    }

    /// Checks if a contract is permanently registered.
    pub fn is_contract_registered(&self, contract_id: ContractId) -> bool {
        self.in_memory_contracts.contains_key(&contract_id)
    }

    /// Returns the account body by its key.
    pub fn get_account_body_by_account_key(
        &self,
        account_key: AccountKey,
    ) -> Option<RMAccountBody> {
        self.in_memory_accounts.get(&account_key).cloned()
    }

    /// Returns the contract body by its identifier.
    pub fn get_contract_body_by_contract_id(
        &self,
        contract_id: ContractId,
    ) -> Option<RMContractBody> {
        self.in_memory_contracts.get(&contract_id).cloned()
    }

    /// Returns the account key by its rank.
    pub fn get_account_key_by_rank(&self, rank: Rank) -> Option<AccountKey> {
        self.in_memory_account_ranks.get(&rank).cloned()
    }

    /// Returns the contract id by its rank.
    pub fn get_contract_id_by_rank(&self, rank: Rank) -> Option<ContractId> {
        self.in_memory_contract_ranks.get(&rank).cloned()
    }

    /// Returns the account body by its rank.
    pub fn get_account_body_by_rank(&self, rank: Rank) -> Option<RMAccountBody> {
        self.in_memory_account_ranks
            .get(&rank)
            .and_then(|account_key| self.in_memory_accounts.get(account_key).cloned())
    }

    /// Returns the contract body by its rank.
    pub fn get_contract_body_by_rank(&self, rank: Rank) -> Option<RMContractBody> {
        self.in_memory_contract_ranks
            .get(&rank)
            .and_then(|contract_id| self.in_memory_contracts.get(contract_id).cloned())
    }

    /// Returns the rank by its account key.
    ///
    /// NOTE: Used by the Engine.
    pub fn get_rank_by_account_key(&self, account_key: AccountKey) -> Option<Rank> {
        self.in_memory_account_ranks
            .iter()
            .find(|(_, key)| *key == &account_key)
            .map(|(rank, _)| *rank)
    }

    /// Returns the rank by its contract id.
    ///
    /// NOTE: Used by the Engine.
    pub fn get_rank_by_contract_id(&self, contract_id: ContractId) -> Option<Rank> {
        self.in_memory_contract_ranks
            .iter()
            .find(|(_, key)| *key == &contract_id)
            .map(|(rank, _)| *rank)
    }

    /// Returns the account by its key.
    pub fn get_account_by_key(&self, account_key: AccountKey) -> Option<Account> {
        // 1 Get the account body by its key.
        let account_body = self.get_account_body_by_account_key(account_key)?;

        // 2 Get the rank by its account key.
        let rank = self.get_rank_by_account_key(account_key)?;

        // 2 Convert the account key to a point.
        let account_key: Point = Point::from_slice(&account_key).ok()?;

        // 3 Construct the account.
        let account = Account::new(
            account_key,
            Some(account_body.registery_index),
            Some(rank as u32),
        )?;

        // 4 Return the account.
        Some(account)
    }

    /// Returns the contract by its id.
    pub fn get_contract_by_contract_id(&self, contract_id: ContractId) -> Option<Contract> {
        // 1 Get the contract body by its id.
        let contract_body = self.get_contract_body_by_contract_id(contract_id)?;

        // 2 Get the rank by its contract id.
        let rank = self.get_rank_by_contract_id(contract_id)?;

        // 3 Construct the contract.
        let contract = Contract::new(
            contract_id,
            contract_body.registery_index,
            Some(rank as u32),
        );

        // 4 Return the contract.
        Some(contract)
    }

    /// Returns the account by its rank.
    pub fn get_account_by_rank(&self, rank: Rank) -> Option<Account> {
        // 1 Get the account key by its rank.
        let account_key = self.get_account_key_by_rank(rank)?;

        // 2 Get the account body by its key.
        let account_body = self.get_account_body_by_account_key(account_key)?;

        // 3 Convert the account key to a point.
        let account_key: Point = Point::from_slice(&account_key).ok()?;

        // 4 Construct the account.
        let account = Account::new(
            account_key,
            Some(account_body.registery_index),
            Some(rank as u32),
        )?;

        // 5 Return the account.
        Some(account)
    }

    /// Returns the contract by its rank.
    pub fn get_contract_by_rank(&self, rank: Rank) -> Option<Contract> {
        // 1 Get the contract id by its rank.
        let contract_id = self.get_contract_id_by_rank(rank)?;

        // 2 Get the contract body by its id.
        let contract_body = self.get_contract_body_by_contract_id(contract_id)?;

        // 4 Construct the contract.
        let contract = Contract::new(
            contract_id,
            contract_body.registery_index,
            Some(rank as u32),
        );

        // 5 Return the contract.
        Some(contract)
    }
    /// Epheremally registers an account.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_account(
        &mut self,
        account_key: AccountKey,
        bls_key: Option<AccountBLSKey>,
        secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,
        flame_config: Option<FlameConfig>,
    ) -> Result<(), RMRegisterAccountError> {
        // 1 Check if the account has just been epheremally registered in the delta.
        if self.delta.is_account_epheremally_registered(account_key) {
            return Err(
                RMRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(account_key),
            );
        }

        // 2 Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(RMRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(account_key));
        }

        // 3 Epheremally register the account in the delta.
        self.delta.epheremally_register_account(
            account_key,
            bls_key,
            secondary_aggregation_key,
            flame_config,
        );

        // 4 Return the result.
        Ok(())
    }

    /// Epheremally registers a contract.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_contract(
        &mut self,
        contract_id: ContractId,
        executable: Executable,
    ) -> Result<(), RMRegisterContractError> {
        // 1 Check if the contract has just been epheremally registered in the delta.
        if self.delta.is_contract_epheremally_registered(contract_id) {
            return Err(
                RMRegisterContractError::ContractHasJustBeenEphemerallyRegistered(contract_id),
            );
        }

        // 2 Check if the contract is already permanently registered.
        if self.is_contract_registered(contract_id) {
            return Err(
                RMRegisterContractError::ContractIsAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // 3 Epheremally register the contract in the delta.
        self.delta
            .epheremally_register_contract(contract_id, executable);

        // 4 Return the result.
        Ok(())
    }

    /// Epheremally increments the call counter of an account.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn epheremally_increment_account_call_counter_by_one(
        &mut self,
        account_key: AccountKey,
        optimized: bool,
    ) -> Result<(), RMIncrementAccountCallCounterError> {
        // 1 If not optimized, check if the account is permanently registered.
        if !optimized {
            if !self.is_account_registered(account_key) {
                return Err(RMIncrementAccountCallCounterError::AccountIsNotRegistered(
                    account_key,
                ));
            }
        }

        // 2 Epheremally increment the call counter delta of the account by one.
        self.delta
            .epheremally_increment_account_call_counter_delta_by_one(account_key);

        // 3 Return the result.
        Ok(())
    }

    /// Epheremally increments the call counter of a contract.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn epheremally_increment_contract_call_counter_by_one(
        &mut self,
        contract_id: ContractId,
        optimized: bool,
    ) -> Result<(), RMIncrementContractCallCounterError> {
        // 1 If not optimized, check if the contract is permanently registered.
        if !optimized {
            if !self.is_contract_registered(contract_id) {
                return Err(
                    RMIncrementContractCallCounterError::ContractIsNotRegistered(contract_id),
                );
            }
        }

        // 2 Epheremally increment the call counter delta of the contract by one.
        self.delta
            .epheremally_increment_contract_call_counter_delta_by_one(contract_id);

        // 3 Return the result.
        Ok(())
    }

    /// Epheremally configures or reconfigures an account.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn epheremally_configure_or_reconfigure_account(
        &mut self,
        account_key: AccountKey,
        bls_key: Option<AccountBLSKey>,
        secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,
        flame_config: Option<FlameConfig>,
    ) -> Result<(), RMReconfigAccountError> {
        // 1 Check if the account is permanently registered.
        let account_body = self
            .in_memory_accounts
            .get(&account_key)
            .ok_or(RMReconfigAccountError::AccountIsNotRegistered(account_key))?;

        // 2 Check if the BLS is to be set.
        if let Some(bls_key) = bls_key {
            // NOTE: We allot BLS key to be set only once, and for that it should not have been set yet.

            // 2.1 Check if the BLS key has already been set.
            if account_body.primary_bls_key.is_some() {
                return Err(RMReconfigAccountError::BLSKeyIsAlreadyPermanentlySet(
                    account_key,
                ));
            }

            // 2.2 Update the BLS key in the delta.
            if let Some(existing_bls_key) = self
                .delta
                .epheremally_set_account_bls_key(account_key, bls_key)
            {
                return Err(RMReconfigAccountError::BLSKeyIsAlreadyEpheremallySet(
                    account_key,
                    existing_bls_key,
                ));
            }
        }

        // 3 Check if the secondary aggregation key is to be set.
        if let Some(secondary_aggregation_key) = secondary_aggregation_key {
            // NOTE: We allow secondary aggregation key to be updated multiple times.

            // 3.1 Update the secondary aggregation key in the delta.
            self.delta
                .epheremally_update_account_secondary_aggregation_key(
                    account_key,
                    secondary_aggregation_key,
                );
        }

        // 4 Check if the flame config is to be set.
        if let Some(flame_config) = flame_config {
            // NOTE: We allow flame config to be updated multiple times.

            // 4.1 Update the flame config in the delta.
            self.delta
                .epheremally_update_account_flame_config(account_key, flame_config);
        }

        // 5 Return the result.
        Ok(())
    }

    /// Reverts the epheremal changes associated with the last execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn rollback_last(&mut self) {
        // Restore the epheremal changes from the backup.
        self.restore_delta();
    }

    /// Clears all epheremal changes from the delta.
    ///
    /// NOTE: Used by the Engine.
    pub fn flush_delta(&mut self) {
        // Clear the epheremal changes from the delta.
        self.delta.flush();

        // Clear the epheremal changes from the backup.
        self.backup_of_delta.flush();
    }

    /// Applies the changes to the registery manager.
    ///
    /// NOTE: Used by the Engine.
    pub fn apply_changes(&mut self) -> Result<(), RMApplyChangesError> {
        // Get the current height of account registery indices.
        let account_registery_index_height = self.in_memory_accounts.len() as u32;

        // Get the current height of contract registery indices.
        let contract_registery_index_height = self.in_memory_contracts.len() as u32;

        // 1 Register new accounts.
        for (index, (account_key, bls_key, secondary_aggregation_key, flame_config)) in
            self.delta.new_accounts_to_register.iter().enumerate()
        {
            // 1.1 Calculate the registery index for the new account.
            let registery_index = account_registery_index_height + index as u32;

            // 1.2 Initial call counter value is set to zero.
            let initial_call_counter = 0u64;

            // 1.3 On-disk insertion.
            {
                // 1.3.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| RMApplyChangesError::AccountTreeOpenError(*account_key, e))?;

                // 1.3.2 Insert the registery index on-disk.
                tree.insert(
                    REGISTERY_INDEX_SPECIAL_DB_KEY,
                    registery_index.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::AccountRegisteryIndexInsertError(
                        *account_key,
                        registery_index,
                        e,
                    )
                })?;

                // 1.3.3 Insert the call counter on-disk.
                tree.insert(
                    CALL_COUNTER_SPECIAL_DB_KEY,
                    initial_call_counter.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::AccountCallCounterInsertError(
                        *account_key,
                        initial_call_counter,
                        e,
                    )
                })?;

                // 1.3.4 Insert the BLS key on-disk if present.
                if let Some(bls_key) = bls_key {
                    tree.insert(BLS_KEY_SPECIAL_DB_KEY, bls_key.as_slice())
                        .map_err(|e| {
                            RMApplyChangesError::AccountBLSKeyInsertError(*account_key, e)
                        })?;
                }

                // 1.3.5 Insert the secondary aggregation key on-disk if present.
                if let Some(secondary_aggregation_key) = secondary_aggregation_key {
                    tree.insert(
                        SECONDARY_AGGREGATION_KEY_SPECIAL_DB_KEY,
                        secondary_aggregation_key.as_slice(),
                    )
                    .map_err(|e| {
                        RMApplyChangesError::AccountSecondaryAggregationKeyInsertError(
                            *account_key,
                            e,
                        )
                    })?;
                }

                // 1.3.6 Insert the flame config on-disk if present.
                if let Some(flame_config) = flame_config {
                    let flame_config_bytes = flame_config.to_db_value_bytes();
                    tree.insert(FLAME_CONFIG_SPECIAL_DB_KEY, flame_config_bytes)
                        .map_err(|e| {
                            RMApplyChangesError::AccountFlameConfigInsertError(*account_key, e)
                        })?;
                }
            }

            // 1.4 In-memory insertion.
            {
                // 1.4.1 Construct the account body.
                let account_body = RMAccountBody::new(
                    registery_index,
                    initial_call_counter,
                    *bls_key,
                    secondary_aggregation_key.clone(),
                    flame_config.clone(),
                );

                // 1.4.2 Insert the account body into the in-memory list.
                self.in_memory_accounts.insert(*account_key, account_body);
            }
        }

        // 2 Register new contracts.
        for (index, (contract_id, executable)) in
            self.delta.new_contracts_to_register.iter().enumerate()
        {
            // 2.1 Calculate the registery index for the new contract.
            let registery_index = contract_registery_index_height + index as u32;

            // 2.2 Initial call counter value is set to zero.
            let initial_call_counter = 0u64;

            // 2.3 Compile the executable to bytes.
            let program_bytes = executable
                .compile()
                .map_err(|e| RMApplyChangesError::ExecutableCompileError(*contract_id, e))?;

            // 2.4 On-disk insertion.
            {
                // 2.4.1 Open the tree for the contract.
                let tree = self
                    .on_disk_contracts
                    .open_tree(contract_id)
                    .map_err(|e| RMApplyChangesError::ContractTreeOpenError(*contract_id, e))?;

                // 2.4.2 Insert the registery index on-disk.
                tree.insert(
                    REGISTERY_INDEX_SPECIAL_DB_KEY,
                    registery_index.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::ContractRegisteryIndexInsertError(
                        *contract_id,
                        registery_index,
                        e,
                    )
                })?;

                // 2.4.3 Insert the call counter on-disk.
                tree.insert(
                    CALL_COUNTER_SPECIAL_DB_KEY,
                    initial_call_counter.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::ContractCallCounterInsertError(
                        *contract_id,
                        initial_call_counter,
                        e,
                    )
                })?;

                // 2.4.4 Insert the program bytes on-disk.
                tree.insert(PROGRAM_BYTES_SPECIAL_DB_KEY, program_bytes.as_slice())
                    .map_err(|e| {
                        RMApplyChangesError::ContractProgramBytesInsertError(*contract_id, e)
                    })?;
            }

            // 2.5 In-memory insertion.
            {
                // 2.5.1 Construct the contract body.
                let contract_body =
                    RMContractBody::new(registery_index, initial_call_counter, executable.clone());

                // 2.5.2 Insert the contract body into the in-memory list.
                self.in_memory_contracts.insert(*contract_id, contract_body);
            }
        }

        // 3 Update account call counters.
        for (account_key, call_counter_delta) in self.delta.updated_account_call_counters.iter() {
            // 3.1 Get the mutable account body from the in-memory list.
            let account_body = self
                .in_memory_accounts
                .get_mut(account_key)
                .ok_or(RMApplyChangesError::AccountNotFoundInMemory(*account_key))?;

            // 3.2 Get the historical call counter.
            let historical_call_counter = account_body.call_counter;

            // 3.3 Calculate the new call counter.
            let new_call_counter = historical_call_counter + *call_counter_delta as u64;

            // 3.4 On-disk update.
            {
                // 3.4.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| RMApplyChangesError::AccountTreeOpenError(*account_key, e))?;

                // 3.4.2 Update the call counter on-disk.
                tree.insert(
                    CALL_COUNTER_SPECIAL_DB_KEY,
                    new_call_counter.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::AccountCallCounterUpdateError(
                        *account_key,
                        new_call_counter,
                        e,
                    )
                })?;
            }

            // 3.5 In-memory update.
            {
                // 3.5.1 Update the call counter.
                account_body.call_counter = new_call_counter;
            }
        }

        // 4 Update contract call counters.
        for (contract_id, call_counter_delta) in self.delta.updated_contract_call_counters.iter() {
            // 4.1 Get the mutable contract body from the in-memory list.
            let contract_body = self
                .in_memory_contracts
                .get_mut(contract_id)
                .ok_or(RMApplyChangesError::ContractNotFoundInMemory(*contract_id))?;

            // 4.2 Get the historical call counter.
            let historical_call_counter = contract_body.call_counter;

            // 4.3 Calculate the new call counter.
            let new_call_counter = historical_call_counter + *call_counter_delta as u64;

            // 4.4 On-disk update.
            {
                // 4.4.1 Open the tree for the contract.
                let tree = self
                    .on_disk_contracts
                    .open_tree(contract_id)
                    .map_err(|e| RMApplyChangesError::ContractTreeOpenError(*contract_id, e))?;

                // 4.4.2 Update the call counter on-disk.
                tree.insert(
                    CALL_COUNTER_SPECIAL_DB_KEY,
                    new_call_counter.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    RMApplyChangesError::ContractCallCounterUpdateError(
                        *contract_id,
                        new_call_counter,
                        e,
                    )
                })?;
            }

            // 4.5 In-memory update.
            {
                // 4.5.1 Update the call counter.
                contract_body.call_counter = new_call_counter;
            }
        }

        // 5 Update account BLS keys.
        for (account_key, bls_key) in self.delta.updated_bls_keys.iter() {
            // 5.1 Get the mutable account body from the in-memory list.
            let mut_account_body = self
                .in_memory_accounts
                .get_mut(account_key)
                .ok_or(RMApplyChangesError::AccountNotFoundInMemory(*account_key))?;

            // 5.2 On-disk update.
            {
                // 5.2.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| RMApplyChangesError::AccountTreeOpenError(*account_key, e))?;

                // 5.2.2 Update the BLS key on-disk.
                tree.insert(BLS_KEY_SPECIAL_DB_KEY, bls_key.as_slice())
                    .map_err(|e| RMApplyChangesError::AccountBLSKeyInsertError(*account_key, e))?;
            }

            // 5.3 In-memory update.
            {
                // 5.3.1 Update the BLS key.
                mut_account_body.primary_bls_key = Some(*bls_key);
            }
        }

        // 6 Update account secondary aggregation keys.
        for (account_key, secondary_aggregation_key) in
            self.delta.updated_secondary_aggregation_keys.iter()
        {
            // 6.1 Get the mutable account body from the in-memory list.
            let mut_account_body = self
                .in_memory_accounts
                .get_mut(account_key)
                .ok_or(RMApplyChangesError::AccountNotFoundInMemory(*account_key))?;

            // 6.2 On-disk update.
            {
                // 6.2.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| RMApplyChangesError::AccountTreeOpenError(*account_key, e))?;

                // 6.2.2 Update the secondary aggregation key on-disk.
                tree.insert(
                    SECONDARY_AGGREGATION_KEY_SPECIAL_DB_KEY,
                    secondary_aggregation_key.as_slice(),
                )
                .map_err(|e| {
                    RMApplyChangesError::AccountSecondaryAggregationKeyInsertError(*account_key, e)
                })?;
            }

            // 6.3 In-memory update.
            {
                // 6.3.1 Update the secondary aggregation key.
                mut_account_body.secondary_aggregation_key =
                    Some(secondary_aggregation_key.clone());
            }
        }

        // 7 Update account flame configs.
        for (account_key, flame_config) in self.delta.updated_flame_configs.iter() {
            // 7.1 Get the mutable account body from the in-memory list.
            let mut_account_body = self
                .in_memory_accounts
                .get_mut(account_key)
                .ok_or(RMApplyChangesError::AccountNotFoundInMemory(*account_key))?;

            // 7.2 On-disk update.
            {
                // 7.2.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| RMApplyChangesError::AccountTreeOpenError(*account_key, e))?;

                // 7.2.2 Serialize the flame config to bytes.
                let flame_config_bytes = flame_config.to_db_value_bytes();

                // 7.2.3 Update the flame config on-disk.
                tree.insert(FLAME_CONFIG_SPECIAL_DB_KEY, flame_config_bytes)
                    .map_err(|e| {
                        RMApplyChangesError::AccountFlameConfigInsertError(*account_key, e)
                    })?;
            }

            // 7.3 In-memory update.
            {
                // 7.3.1 Update the flame config.
                mut_account_body.flame_config = Some(flame_config.clone());
            }
        }

        // 8 Re-rank accounts after all changes.
        {
            let new_ranked_accounts = Self::rank_accounts(&self.in_memory_accounts);
            self.in_memory_account_ranks = new_ranked_accounts;
        }

        // 9 Re-rank contracts after all changes.
        {
            let new_ranked_contracts = Self::rank_contracts(&self.in_memory_contracts);
            self.in_memory_contract_ranks = new_ranked_contracts;
        }

        // 10 Flush the delta.
        self.flush_delta();

        // 11 Return the result.
        Ok(())
    }

    /// Returns the registery manager as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the registery manager JSON object.
        let mut obj = Map::new();

        // 2 Insert the in-memory accounts.
        obj.insert(
            "accounts".to_string(),
            Value::Object(
                self.in_memory_accounts
                    .iter()
                    .map(|(account_key, account_body)| {
                        (hex::encode(account_key), account_body.json())
                    })
                    .collect(),
            ),
        );

        // 3 Insert the in-memory contracts.
        obj.insert(
            "contracts".to_string(),
            Value::Object(
                self.in_memory_contracts
                    .iter()
                    .map(|(contract_id, contract_body)| {
                        (hex::encode(contract_id), contract_body.json())
                    })
                    .collect(),
            ),
        );

        // 4 Return the registery manager JSON object.
        Value::Object(obj)
    }
}

/// Erases the registery manager by db paths.
pub fn erase_registery_manager(chain: Chain) {
    // Accounts db path.
    let accounts_db_path = format!("storage/{}/registery/accounts", chain.to_string());

    // Erase the accounts db path.
    let _ = std::fs::remove_dir_all(accounts_db_path);

    // Contracts db path.
    let contracts_db_path = format!("storage/{}/registery/contracts", chain.to_string());

    // Erase the contracts db path.
    let _ = std::fs::remove_dir_all(contracts_db_path);
}
