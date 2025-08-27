use super::{
    account_registery::account_registery::{AccountRegistery, ACCOUNT_REGISTERY},
    contract_registery::contract_registery::{ContractRegistery, CONTRACT_REGISTERY},
    registery_error::RegisteryConstructionError,
};
use crate::operative::Chain;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Guarded registery.
#[allow(non_camel_case_types)]
pub type REGISTERY = Arc<Mutex<Registery>>;

/// Directory for the account registeries.
pub struct Registery {
    account_registery: ACCOUNT_REGISTERY,
    contract_registery: CONTRACT_REGISTERY,
}

impl Registery {
    pub fn new(chain: Chain) -> Result<REGISTERY, RegisteryConstructionError> {
        // Construct the account registery.
        let account_registery = AccountRegistery::new(chain)
            .map_err(RegisteryConstructionError::AccountRegisteryConstructionError)?;

        // Construct the contract registery.
        let contract_registery = ContractRegistery::new(chain)
            .map_err(RegisteryConstructionError::ContractRegisteryConstructionError)?;

        // Construct the registery.
        let registery = Registery {
            account_registery,
            contract_registery,
        };

        // Guard the registery.
        let guarded_registery = Arc::new(Mutex::new(registery));

        // Return the guarded registery.
        Ok(guarded_registery)
    }

    pub fn account_registery(&self) -> ACCOUNT_REGISTERY {
        Arc::clone(&self.account_registery)
    }

    pub fn contract_registery(&self) -> CONTRACT_REGISTERY {
        Arc::clone(&self.contract_registery)
    }
}
