use crate::constructive::entity::account::account::ext::validate_account::validate_account_error::AccountValidateAccountError;
use crate::constructive::entity::account::root_account::ext::validate_root_account::validate_root_account_error::RootAccountValidateRootAccountError;

/// Errors from [`Move::validate_overall`].
#[derive(Debug, Clone)]
pub enum MoveValidateOverallError {
    UnregisteredRootAccountNotAllowedError,
    ValidateRootAccountError(RootAccountValidateRootAccountError),
    ValidateAccountError(AccountValidateAccountError),
    ValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    FromAccountNotFoundInCoinManagerError([u8; 32]),
    InsufficientBalanceForMoveError {
        account_key: [u8; 32],
        required: u64,
        available: u64,
    },
}
