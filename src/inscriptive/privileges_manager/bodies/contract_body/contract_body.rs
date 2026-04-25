use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;

/// A struct for containing the privileges of a contract.
#[derive(Clone)]
pub struct PrivilegesManagerContractBody {
    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The immutability of the contract.
    pub immutability: bool,

    // The ocuupancy tax exemptions of the contract.
    pub tax_exemptions: Exemption,
}

impl PrivilegesManagerContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(
        liveness_flag: LivenessFlag,
        immutability: bool,
        tax_exemptions: Exemption,
    ) -> PrivilegesManagerContractBody {
        PrivilegesManagerContractBody {
            liveness_flag,
            immutability,
            tax_exemptions,
        }
    }
}
