use crate::constructive::entry::entry_kinds::swapout::ext::signature::bls_verify::error::bls_verify_error::SwapoutBLSVerifyError;

#[derive(Debug, Clone)]
pub enum SwapoutValidateOverallError {
    ValidateBLSSignatureError(SwapoutBLSVerifyError),
    ValidateRootAccountError,
    ValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    SwapoutUnknownPinlessSelfNotSupportedYetError,
    SwapoutDefaultPinlessSelfLocationMustBeAbsentError,
    InsufficientBalance {
        requested_amount: u32,
        account_balance: u64,
    },
    UnableToReadAccountBalance,
}
