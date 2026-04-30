use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;

#[derive(Debug, Clone)]
pub enum SwapoutExecutionError {
    ValidatePinlessSelfScriptpubkeyError,
    CoinManagerAccountBalanceDownError(CMAccountBalanceDownError),
    AmountPlusFeesOverflow,
}
