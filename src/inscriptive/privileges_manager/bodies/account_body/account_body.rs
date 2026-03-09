use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;

/// A struct for containing the privileges of an account.
#[derive(Clone)]
pub struct PrivilegesManagerAccountBody {
    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The hierarchy of the account.
    pub hierarchy: AccountHierarchy,

    // The transaction fee exemptions of the account.
    pub txfee_exemptions: Exemption,

    // Whether the account can deploy liquidity (liquidity provider).
    pub can_deploy_liquidity: TimedSwitchBool,

    // Whether the account can deploy a contract (developer).
    pub can_deploy_contract: TimedSwitchBool,

    // The last activity timestamp of the account.
    pub last_activity_timestamp: u64,
}

impl PrivilegesManagerAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(
        liveness_flag: LivenessFlag,
        hierarchy: AccountHierarchy,
        txfee_exemptions: Exemption,
        can_deploy_liquidity: TimedSwitchBool,
        can_deploy_contract: TimedSwitchBool,
        last_activity_timestamp: u64,
    ) -> PrivilegesManagerAccountBody {
        PrivilegesManagerAccountBody {
            liveness_flag,
            hierarchy,
            txfee_exemptions,
            can_deploy_liquidity,
            can_deploy_contract,
            last_activity_timestamp,
        }
    }
}
