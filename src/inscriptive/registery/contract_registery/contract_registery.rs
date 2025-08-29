use super::contract_registery_error::{
    ContractRegisteryConstructionError, ContractRegisteryRegisterError,
    ContractRegisterySaveAllError,
};
use crate::{
    constructive::{
        entity::contract::contract::Contract, valtype::val::short_val::short_val::ShortVal,
    },
    inscriptive::registery::contract_registery::contract_registery_error::ContractRegisteryIncrementCallCounterError,
    operative::Chain,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded registery of contracts.
#[allow(non_camel_case_types)]
pub type CONTRACT_REGISTERY = Arc<Mutex<ContractRegistery>>;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Registery index of a contract for efficient referencing (from 1 to U32::MAX).
#[allow(non_camel_case_types)]
type REGISTERY_INDEX = u32;

/// Call counter of a contract used to rank contracts.
#[allow(non_camel_case_types)]
type CALL_COUNTER = u64;

/// Rank integer representing the rank position of a contract (from 1 to U32::MAX).
#[allow(non_camel_case_types)]
type RANK = u32;

/// In-block local call counter of a contract used to increment the call counter.
#[allow(non_camel_case_types)]
type IN_BLOCK_CALL_COUNTER = u16;

/// Body of a contract.
struct ContractBody {
    pub registery_index: u32,
    pub call_counter: u64,
}

impl ContractBody {
    /// Updates the call counter of a contract.
    pub fn update_call_counter(&mut self, new_call_counter: u64) {
        self.call_counter = new_call_counter;
    }
}

/// Directory for storing contracts and their call counters.
/// There are two in-memory lists, one by registery index and one by rank.
pub struct ContractRegistery {
    // In-memory list of contracts by registery index.
    in_memory_contracts: HashMap<CONTRACT_ID, ContractBody>,
    // In-memory list of contracts by rank.
    in_memory_ranks: HashMap<RANK, CONTRACT_ID>,

    // In-storage db for storing the contracts and their call counters.
    on_disk_db: sled::Db,

    // Ephemeral states
    epheremal_contracts_to_register: Vec<CONTRACT_ID>,
    epheremal_contracts_to_increment: HashMap<CONTRACT_ID, IN_BLOCK_CALL_COUNTER>,

    // Backups
    backup_of_ephemeral_contracts_to_register: Vec<CONTRACT_ID>,
    backup_of_ephemeral_contracts_to_increment: HashMap<CONTRACT_ID, IN_BLOCK_CALL_COUNTER>,
}

impl ContractRegistery {
    pub fn new(chain: Chain) -> Result<CONTRACT_REGISTERY, ContractRegisteryConstructionError> {
        // Construct the contracts db path.
        let contract_registery_path =
            format!("{}/{}/{}", "db", chain.to_string(), "registery/contract");

        // Open the contracts db.
        let contract_registery_db = {
            sled::open(contract_registery_path)
                .map_err(ContractRegisteryConstructionError::ContractsDBOpenError)?
        };

        // Initialize the in-memory list of contracts.
        let mut contracts = HashMap::<CONTRACT_ID, ContractBody>::new();

        // Iterate over all items in the db.
        for tree_name in contract_registery_db.tree_names() {
            // Convert the tree name to a contract id.
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                ContractRegisteryConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Initialize the registery index and call counter.
            let mut registery_index = 0;

            // Initialize the call counter.
            let mut call_counter = 0;

            // Open the contract registery tree.
            let tree = contract_registery_db.open_tree(&tree_name).map_err(|e| {
                ContractRegisteryConstructionError::ContractRegisteryTreeOpenError(contract_id, e)
            })?;

            // Iterate over all items in the contract registery tree.
            for item in tree.iter() {
                // Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(
                            ContractRegisteryConstructionError::ContractRegisteryTreeOpenError(
                                contract_id,
                                e,
                            ),
                        );
                    }
                };

                // Convert the key to a key byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    ContractRegisteryConstructionError::InvalidContractIDBytes(key.to_vec())
                })?;

                // Match the key byte.
                match key_byte[0] {
                    // 0x00 key byte represents the registery index.
                    0u8 => {
                        // Convert the value to a registery index bytes.
                        let registery_index_bytes: [u8; 4] =
                            value.as_ref().try_into().map_err(|_| {
                                ContractRegisteryConstructionError::InvalidRegisteryIndexBytes(
                                    value.to_vec(),
                                )
                            })?;

                        // Convert the registery index bytes to a registery index.
                        registery_index = u32::from_le_bytes(registery_index_bytes);
                    }

                    // 0x01 key byte represents the call counter.
                    1u8 => {
                        // Convert the value to a call counter bytes.
                        let call_counter_bytes: [u8; 8] =
                            value.as_ref().try_into().map_err(|_| {
                                ContractRegisteryConstructionError::InvalidCallCounterBytes(
                                    value.to_vec(),
                                )
                            })?;

                        // Convert the call counter bytes to a call counter.
                        call_counter = u64::from_le_bytes(call_counter_bytes);
                    }

                    _ => {
                        return Err(ContractRegisteryConstructionError::InvalidKeyByte(
                            key.to_vec(),
                        ));
                    }
                }

                // Construct the contract body.
                let contract_body = ContractBody {
                    registery_index,
                    call_counter,
                };

                // Insert the contract body into the in-memory list of contracts.
                contracts.insert(contract_id, contract_body);
            }
        }

        // Rank the contracts by call counter (descending) and registry index (ascending as tiebreaker).
        let ranked = Self::rank_contracts(&contracts);

        // Construct the contract registery.
        let contract_registery = ContractRegistery {
            in_memory_contracts: contracts,
            in_memory_ranks: ranked,
            on_disk_db: contract_registery_db,
            epheremal_contracts_to_register: Vec::new(),
            epheremal_contracts_to_increment: HashMap::new(),
            backup_of_ephemeral_contracts_to_register: Vec::new(),
            backup_of_ephemeral_contracts_to_increment: HashMap::new(),
        };

        // Guard the contract registery.
        let guarded_contract_registery = Arc::new(Mutex::new(contract_registery));

        // Return the guarded contract registery.
        Ok(guarded_contract_registery)
    }

    /// Ranks contracts by call counter (descending) and registry index (ascending as tiebreaker).
    /// Returns a HashMap where keys are ranks starting from 1.
    fn rank_contracts(
        contracts: &HashMap<CONTRACT_ID, ContractBody>,
    ) -> HashMap<RANK, CONTRACT_ID> {
        // Convert to vector for sorting
        let mut contract_vec: Vec<(&CONTRACT_ID, &ContractBody)> = contracts.iter().collect();

        // Sort by call counter (descending), then by registry index (ascending) as tiebreaker
        contract_vec.sort_by(|a, b| {
            // Primary sort: call counter (descending)
            b.1.call_counter
                .cmp(&a.1.call_counter)
                // Secondary sort: registry index (ascending) as tiebreaker
                .then(a.1.registery_index.cmp(&b.1.registery_index))
        });

        // Convert to ranked HashMap with ranks starting from 1
        let mut ranked_contracts = HashMap::<RANK, CONTRACT_ID>::new();
        for (index, (contract_id, _)) in contract_vec.into_iter().enumerate() {
            let rank = (index + 1) as RANK; // Start from 1
            ranked_contracts.insert(rank, *contract_id);
        }

        ranked_contracts
    }

    /// Checks if a contract is registered.
    pub fn is_contract_registered(&self, contract_id: CONTRACT_ID) -> bool {
        // Try from ephemeral states first.
        if self.epheremal_contracts_to_register.contains(&contract_id) {
            return true;
        }

        // Try from in-memory states next.
        self.in_memory_contracts.contains_key(&contract_id)
    }

    /// Returns the rank of a contract by its ID.
    pub fn get_rank_by_contract_id(&self, contract_id: CONTRACT_ID) -> Option<RANK> {
        // Iterate ranked list and return the rank of the contract ID.
        for (rank, id) in self.in_memory_ranks.iter() {
            // If the ID matches, return the rank.
            if id == &contract_id {
                return Some(*rank);
            }
        }

        // If the contract is not found, return None.
        None
    }

    /// Returns the contract ID by its rank.
    pub fn get_contract_id_by_rank(&self, rank: RANK) -> Option<CONTRACT_ID> {
        // Return the contract ID by the rank.
        self.in_memory_ranks.get(&rank).cloned()
    }

    /// Returns the contract by its ID.
    pub fn get_contract_info_by_contract_id(
        &self,
        contract_id: CONTRACT_ID,
    ) -> Option<(REGISTERY_INDEX, CALL_COUNTER, RANK)> {
        // Return the contract body by the contract ID.
        let contract_body = self.in_memory_contracts.get(&contract_id)?;

        let registery_index = contract_body.registery_index;
        let call_counter = contract_body.call_counter;
        let rank = self.get_rank_by_contract_id(contract_id)?;
        Some((registery_index, call_counter, rank))
    }

    /// Returns the contract by its key.
    pub fn get_contract_by_contract_key(&self, contract_id: CONTRACT_ID) -> Option<Contract> {
        let contract_body = self.in_memory_contracts.get(&contract_id)?;
        let rank = self.get_rank_by_contract_id(contract_id)?;
        let registery_index = contract_body.registery_index;
        Some(Contract {
            contract_id,
            registery_index: ShortVal::new(registery_index as u32),
            rank: Some(ShortVal::new(rank as u32)),
        })
    }

    /// Returns the contract by its rank.
    pub fn get_contract_by_rank(&self, rank: RANK) -> Option<Contract> {
        // Return the contract ID by the rank.
        let contract_id = self.in_memory_ranks.get(&rank).cloned()?;
        let contract_body = self.in_memory_contracts.get(&contract_id)?;
        let registery_index = contract_body.registery_index;
        Some(Contract {
            contract_id,
            registery_index: ShortVal::new(registery_index as u32),
            rank: Some(ShortVal::new(rank as u32)),
        })
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.backup_of_ephemeral_contracts_to_register =
            self.epheremal_contracts_to_register.clone();
        self.backup_of_ephemeral_contracts_to_increment =
            self.epheremal_contracts_to_increment.clone();
    }

    /// Prepares the registery for the next execution.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Epheremally registers a contract to the registery.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn register_contract(
        &mut self,
        contract_id: CONTRACT_ID,
    ) -> Result<(), ContractRegisteryRegisterError> {
        // If the contract is already registered, return an error.
        if self.is_contract_registered(contract_id) {
            return Err(
                ContractRegisteryRegisterError::ContractAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // If the contract is already pushed to epheremal list, return an error.
        if self.epheremal_contracts_to_register.contains(&contract_id) {
            return Err(
                ContractRegisteryRegisterError::ContractAlreadyEphemerallyRegistered(contract_id),
            );
        }

        // Push the contract to the ephemeral list.
        self.epheremal_contracts_to_register.push(contract_id);

        // Return the result.
        Ok(())
    }

    /// Epheremally increments the call counter of a contract.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn increment_contract_call_counter(
        &mut self,
        contract_id: CONTRACT_ID,
        optimized: bool,
    ) -> Result<(), ContractRegisteryIncrementCallCounterError> {
        // If not optimized, check if the contract is registered.
        if !optimized {
            if !self.is_contract_registered(contract_id) {
                return Err(
                    ContractRegisteryIncrementCallCounterError::ContractNotRegistered(contract_id),
                );
            }
        }

        // Try to get the in-block call counter from the epheremal list.
        let in_block_call_counter = match self.epheremal_contracts_to_increment.get(&contract_id) {
            Some(value) => *value,
            None => 0,
        };

        // Increment the call counter.
        let new_in_block_call_counter = in_block_call_counter + 1;

        // Insert the new call counter into the epheremal list.
        self.epheremal_contracts_to_increment
            .insert(contract_id, new_in_block_call_counter);

        // Return the result.
        Ok(())
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.epheremal_contracts_to_register =
            self.backup_of_ephemeral_contracts_to_register.clone();
        self.epheremal_contracts_to_increment =
            self.backup_of_ephemeral_contracts_to_increment.clone();
    }

    /// Restores the last ephemeral state.
    pub fn rollback_last(&mut self) {
        self.restore_ephemeral_states();
    }

    /// Clears all ephemeral states.
    pub fn rollback_all(&mut self) {
        // Clear the ephemeral states.
        self.epheremal_contracts_to_register.clear();
        self.epheremal_contracts_to_increment.clear();

        // Clear the backup.
        self.backup_of_ephemeral_contracts_to_register.clear();
        self.backup_of_ephemeral_contracts_to_increment.clear();
    }

    /// Returns the height of the registery index.
    fn registery_index_height(&self) -> u32 {
        self.in_memory_contracts.len() as u32
    }

    /// Saves all ephemeral states to in-memory and on-disk.
    pub fn save_all(&mut self) -> Result<(), ContractRegisterySaveAllError> {
        // Calculate the registery index.
        let registery_index_height = self.registery_index_height();

        // Register the contracts.
        for (index, contract_id) in self.epheremal_contracts_to_register.iter().enumerate() {
            // Calculate the registery index.
            let registery_index = registery_index_height + index as u32;

            // Initial call counter value is set to zero.
            let initial_call_counter = 0;

            // Save in-memory:
            {
                // Construct the contract body.
                let contract_body = ContractBody {
                    registery_index,
                    call_counter: initial_call_counter,
                };

                // Insert the contract body into the in-memory list.
                self.in_memory_contracts.insert(*contract_id, contract_body);
            }

            // Save on-disk:
            {
                // Open the tree for the contract.
                let on_disk_contract_tree =
                    self.on_disk_db.open_tree(contract_id).map_err(|e| {
                        ContractRegisterySaveAllError::UnableToOpenContractTree(*contract_id, e)
                    })?;

                // Get the registery index bytes.
                let registery_index_bytes = registery_index.to_le_bytes().to_vec();

                // Insert the registery index into the tree.
                // 0x00 key byte represents the registery index.
                on_disk_contract_tree
                    .insert([0x00u8; 1], registery_index_bytes.to_vec())
                    .map_err(|e| {
                        ContractRegisterySaveAllError::UnableToInsertRegisteryIndex(*contract_id, e)
                    })?;

                // Fresh new call counter bytes.
                let initial_call_counter_bytes = initial_call_counter.to_le_bytes().to_vec();

                // Insert the call counter into the tree.
                // 0x01 key byte represents the call counter.
                on_disk_contract_tree
                    .insert([0x01u8; 1], initial_call_counter_bytes)
                    .map_err(|e| {
                        ContractRegisterySaveAllError::UnableToInsertCallCounter(*contract_id, e)
                    })?;
            }
        }

        // Increment the call counter of the contracts.
        for (contract_id, in_block_call_counter) in self.epheremal_contracts_to_increment.iter() {
            // Get the mutable contract body from the in-memory list.
            let in_memory_contract_body = self.in_memory_contracts.get_mut(contract_id).ok_or(
                ContractRegisterySaveAllError::UnableToGetContractCallCounter(*contract_id),
            )?;

            // Get the historical call counter.
            let historical_call_counter = in_memory_contract_body.call_counter;

            // Calculate the new call counter.
            let new_call_counter = historical_call_counter + *in_block_call_counter as u64;

            // Save in-memory:
            {
                // Update the call counter.
                in_memory_contract_body.update_call_counter(new_call_counter);
            }

            // Save on-disk:
            {
                // Open the tree for the contract.
                let on_disk_contract_tree =
                    self.on_disk_db.open_tree(contract_id).map_err(|e| {
                        ContractRegisterySaveAllError::UnableToOpenContractTree(*contract_id, e)
                    })?;

                // Get the call counter bytes.
                let new_call_counter_bytes = new_call_counter.to_le_bytes().to_vec();

                // Insert the new call counter into the tree.
                // 0x01 key byte represents the call counter.
                on_disk_contract_tree
                    .insert([0x01u8; 1], new_call_counter_bytes)
                    .map_err(|e| {
                        ContractRegisterySaveAllError::UnableToInsertCallCounter(*contract_id, e)
                    })?;
            }
        }

        // Rank the contracts by call counter (descending) and registry index (ascending as tiebreaker).
        let new_ranked_contracts = Self::rank_contracts(&self.in_memory_contracts);

        // Update the ranked list.
        self.in_memory_ranks = new_ranked_contracts;

        // Return the result.
        Ok(())
    }
}
