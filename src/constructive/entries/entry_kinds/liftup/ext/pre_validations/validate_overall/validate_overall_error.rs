use crate::constructive::entity::account::root_account::ext::validate_root_account::validate_root_account_error::RootAccountValidateRootAccountError;
use crate::constructive::entry::entry_kinds::liftup::ext::pre_validations::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;
use crate::constructive::entry::entry_kinds::liftup::ext::signature::bls_verify::error::bls_verify_error::LiftupBLSVerifyError;

/// Errors from [`Liftup::validate_overall`]: root account, then `Target`, then lifts.
#[derive(Debug, Clone)]
pub enum LiftupValidateOverallError {
    ValidateBLSSignatureError(LiftupBLSVerifyError),
    ValidateRootAccountError(RootAccountValidateRootAccountError),
    ValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    ValidateLiftsError(LiftupValidateLiftsError),
}
