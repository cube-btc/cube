use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::account_spend_credit::account_spend_credit::AccountSpendCredit;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;

/// A struct for containing the privileges of an account.
#[derive(Clone)]
pub struct PMAccountBody {
    // The hierarchy of the account.
    pub hierarchy: AccountHierarchy,

    // The liveness flag of the account.
    pub liveness_flag: LivenessFlag,

    // The last activity timestamp of the account.
    pub last_activity_timestamp: u64,

    // The spending credit of the account (VIP cards).
    pub spend_credit: AccountSpendCredit,
}
