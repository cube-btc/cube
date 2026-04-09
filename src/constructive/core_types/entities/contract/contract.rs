use crate::constructive::entity::contract::deployed_contract::deployed_contract::DeployedContract;
use crate::constructive::entity::contract::undeployed_contract::undeployed_contract::UndeployedContract;
use crate::executive::executable::executable::Executable;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
/// A contract is a program that can be called by an account.
pub enum Contract {
    // A deployed contract, registered with the 'Registery Manager'.
    DeployedContract(DeployedContract),

    // An undeployed contract, not registered or yet to be registered with the 'Registery Manager'.
    UndeployedContract(UndeployedContract),
}

impl Contract {
    pub fn new_deployed_contract(
        contract_id: [u8; 32],
        executable: Executable,
        registery_index: u64,
    ) -> Contract {
        // 1 Construct the deployed contract.
        let deployed_contract = DeployedContract::new(contract_id, executable, registery_index);

        // 2 Return the deployed contract.
        Contract::DeployedContract(deployed_contract)
    }

    pub fn new_undeployed_contract(contract_id: [u8; 32], executable: Executable) -> Contract {
        // 1 Construct the undeployed contract.
        let undeployed_contract = UndeployedContract::new(contract_id, executable);

        // 2 Return the undeployed contract.
        Contract::UndeployedContract(undeployed_contract)
    }

    /// Returns whether the contract is deployed.
    pub fn is_deployed(&self) -> bool {
        match self {
            // The contract is deployed.
            Contract::DeployedContract(_) => true,

            // The contract is not deployed.
            Contract::UndeployedContract(_) => false,
        }
    }

    /// Returns the contract id.
    pub fn contract_id(&self) -> [u8; 32] {
        match self {
            // The contract is deployed.
            Contract::DeployedContract(deployed_contract) => deployed_contract.contract_id,

            // The contract is not deployed.
            Contract::UndeployedContract(undeployed_contract) => undeployed_contract.contract_id,
        }
    }

    /// Returns the executable of the contract.
    pub fn executable(&self) -> &Executable {
        match self {
            // The contract is deployed.
            Contract::DeployedContract(deployed_contract) => &deployed_contract.executable,

            // The contract is not deployed.
            Contract::UndeployedContract(undeployed_contract) => &undeployed_contract.executable,
        }
    }

    /// Returns the methods length of the contract.
    pub fn methods_len(&self) -> usize {
        self.executable().methods_len()
    }

    /// Returns the registery index of the contract.
    pub fn registery_index(&self) -> Option<u64> {
        match self {
            Contract::DeployedContract(deployed_contract) => {
                // Return the registery index.
                Some(deployed_contract.registery_index)
            }

            // The contract is not deployed.
            Contract::UndeployedContract(_) => None,
        }
    }

    /// Returns the contract as a JSON object.
    pub fn json(&self) -> Value {
        match self {
            // The contract is deployed.
            Contract::DeployedContract(deployed_contract) => deployed_contract.json(),

            // The contract is not deployed.
            Contract::UndeployedContract(undeployed_contract) => undeployed_contract.json(),
        }
    }
}

impl PartialEq for Contract {
    fn eq(&self, other: &Self) -> bool {
        self.contract_id() == other.contract_id()
    }
}

impl Eq for Contract {}
