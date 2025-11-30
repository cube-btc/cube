use crate::executive::program::compiler::compiler::ProgramCompiler;
use crate::executive::program::program::Program;
use crate::inscriptive::registery_manager::bodies::account_body::account_body::RMAccountBody;
use crate::inscriptive::registery_manager::bodies::account_body::flame_config::flame_config::FlameConfig;
use crate::inscriptive::registery_manager::bodies::contract_body::contract_body::RMContractBody;
use crate::inscriptive::registery_manager::delta::delta::RMDelta;
use crate::inscriptive::registery_manager::errors::construction_error::RMConstructionError;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account Key.
type AccountKey = [u8; 32];

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
            let mut bls_key = [0u8; 48];

            // 4.4 Initialize the secondary aggregation key to None.
            let mut secondary_aggregation_key: Option<Vec<u8>> = None;

            // 4.5 Initialize the flame config to a fresh new flame config.
            let mut flame_config = FlameConfig::fresh_new();

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
                        // Get the primary BLS key bytes.
                        let bls_key_bytes: [u8; 48] = value.as_ref().try_into().map_err(|_| {
                            RMConstructionError::UnableToDeserializeAccountPrimaryBLSKeyBytesFromTreeValue(account_key, value.to_vec())
                        })?;

                        // Update the primary BLS key.
                        bls_key = bls_key_bytes;
                    }
                    // 0x04 key byte represents the secondary aggregation key.
                    SECONDARY_AGGREGATION_KEY_SPECIAL_DB_KEY => {
                        // Convert the value to a secondary aggregation key bytes.
                        let secondary_aggregation_key_bytes: Vec<u8> = value.as_ref().to_vec();

                        // If the secondary aggregation key bytes are not empty, update the secondary aggregation key.
                        if secondary_aggregation_key_bytes.len() > 0 {
                            secondary_aggregation_key = Some(secondary_aggregation_key_bytes);
                        }
                    }
                    // 0x05 key byte represents the flame config.
                    FLAME_CONFIG_SPECIAL_DB_KEY => {
                        // Get the flame config bytes.
                        let flame_config_bytes: Vec<u8> = value.as_ref().to_vec();

                        // Deserialize the flame config from bytes.
                        flame_config = FlameConfig::from_db_value_bytes(&flame_config_bytes).ok_or(RMConstructionError::UnableToDeserializeAccountFlameConfigBytesFromTreeValue(account_key, value.to_vec()))?;
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

            // 5.4 Construct a placeholder program.
            let mut program = Program::placeholder_program();

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
                        // Convert the value to a program bytes.
                        let program_bytes: Vec<u8> = value.as_ref().to_vec();

                        // Decompile the program from bytecode and update the program.
                        program =
                            Program::decompile(&mut program_bytes.into_iter()).map_err(|e| {
                                RMConstructionError::ContractProgramDecompileError(contract_id, e)
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
            let contract_body = RMContractBody::new(registery_index, call_counter, program);

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
}
