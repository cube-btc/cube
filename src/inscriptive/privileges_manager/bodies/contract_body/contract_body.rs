use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_long_val::timed_switch_long_val::TimedSwitchLongVal;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;

/// A struct for containing the privileges of a contract.
#[derive(Clone)]
pub struct PrivilegesManagerContractBody {
    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The immutability of the contract.
    pub immutability: bool,

    // The ocuupancy tax exemptions of the contract.
    pub tax_exemptions: Exemption,

    // The storage bytes limit of the contract.
    pub storage_limit: TimedSwitchLongVal,

    // The last activity timestamp of the contract.
    pub last_activity_timestamp: u64,
}

impl PrivilegesManagerContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(
        liveness_flag: LivenessFlag,
        immutability: bool,
        tax_exemptions: Exemption,
        storage_limit: TimedSwitchLongVal,
        last_activity_timestamp: u64,
    ) -> PrivilegesManagerContractBody {
        PrivilegesManagerContractBody {
            liveness_flag,
            immutability,
            tax_exemptions,
            storage_limit,
            last_activity_timestamp,
        }
    }
}
