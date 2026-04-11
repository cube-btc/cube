use crate::constructive::entity::account::root_account::ext::validate_root_account::validate_root_account_error::RootAccountValidateRootAccountError;
use crate::constructive::entry::entry_kinds::liftup::ext::validations::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;

/// Errors from [`Liftup::validate_overall`]: root account, then `Target`, then lifts.
#[derive(Debug, Clone)]
pub enum LiftupValidateOverallError {
    ValidateRootAccountError(RootAccountValidateRootAccountError),
    ValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    ValidateLiftsError(LiftupValidateLiftsError),
}
