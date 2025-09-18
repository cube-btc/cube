use crate::inscriptive::coin_holder::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use std::collections::HashMap;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A struct for containing state differences to be applied.
#[derive(Clone)]
pub struct CHContractDelta {
    // New contracts to register.
    pub new_contracts_to_register: Vec<CONTRACT_ID>,

    // New accounts to allocate for a given contract.
    pub allocs_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,

    // Existing accounts to deallocate for a given contract.
    pub deallocs_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,

    // Updated contract balances for a given contract.
    pub updated_contract_balances: HashMap<CONTRACT_ID, SATOSHI_AMOUNT>,

    // Updated shadow spaces for a given contract.
    pub updated_shadow_spaces: HashMap<CONTRACT_ID, ShadowSpace>,
}

impl CHContractDelta {
    /// Constructs a fresh new contract delta.
    pub fn new() -> Self {
        Self {
            new_contracts_to_register: Vec::new(),
            allocs_list: HashMap::new(),
            deallocs_list: HashMap::new(),
            updated_contract_balances: HashMap::new(),
            updated_shadow_spaces: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_contracts_to_register.clear();
        self.allocs_list.clear();
        self.deallocs_list.clear();
        self.updated_contract_balances.clear();
        self.updated_shadow_spaces.clear();
    }
}
