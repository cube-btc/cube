use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_long_val::timed_switch_long_val::TimedSwitchLongVal;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::contract_tax_privileges::contract_tax_privileges::ContractTaxPrivileges;

/// A struct for containing the privileges of a contract.
#[derive(Clone)]
pub struct PrivilegesManagerContractBody {
    // The immutability of the contract.
    pub immutability: TimedSwitchBool,

    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The last activity timestamp of the contract.
    pub last_activity_timestamp: u64,

    // The tax privileges of the contract.
    pub tax_privileges: ContractTaxPrivileges,

    // The storage bytes limit of the contract.
    pub storage_limit: TimedSwitchLongVal,
}

impl PrivilegesManagerContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(
        immutability: TimedSwitchBool,
        liveness_flag: LivenessFlag,
        last_activity_timestamp: u64,
        tax_privileges: ContractTaxPrivileges,
        storage_limit: TimedSwitchLongVal,
    ) -> PrivilegesManagerContractBody {
        PrivilegesManagerContractBody {
            immutability,
            liveness_flag,
            last_activity_timestamp,
            tax_privileges,
            storage_limit,
        }
    }
}
