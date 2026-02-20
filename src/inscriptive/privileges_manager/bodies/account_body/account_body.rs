use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::account_txfee_privileges::account_txfee_privileges::AccountTxFeePrivileges;
use crate::inscriptive::privileges_manager::elements::account_transacting_limits::account_transacting_limits::AccountTransactingLimits;

/// A struct for containing the privileges of an account.
#[derive(Clone)]
pub struct PrivilegesManagerAccountBody {
    // The hierarchy of the account.
    pub hierarchy: AccountHierarchy,

    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The last activity timestamp of the account.
    pub last_activity_timestamp: u64,

    // The transaction fee privileges of the account (credit, discount, etc.).
    pub txfee_privileges: AccountTxFeePrivileges,

    // The account transacting limits (number of entries and ops per period).
    pub transacting_limits: AccountTransactingLimits,

    // Whether the account can deploy liquidity (liquidity provider).
    pub can_deploy_liquidity: bool,

    // Whether the account can deploy a contract (developer).
    pub can_deploy_contract: bool,
}

impl PrivilegesManagerAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(
        hierarchy: AccountHierarchy,
        liveness_flag: LivenessFlag,
        last_activity_timestamp: u64,
        txfee_privileges: AccountTxFeePrivileges,
        transacting_limits: AccountTransactingLimits,
        can_deploy_liquidity: bool,
        can_deploy_contract: bool,
    ) -> PrivilegesManagerAccountBody {
        PrivilegesManagerAccountBody {
            hierarchy,
            liveness_flag,
            last_activity_timestamp,
            txfee_privileges,
            transacting_limits,
            can_deploy_liquidity,
            can_deploy_contract,
        }
    }
}
