use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;

/// Errors associated with executing a `Liftup` entry.
#[derive(Debug, Clone)]
pub enum LiftupExecutionError {
    LiftupValidationError,
    CoinManagerIncreaseBalanceError(CMAccountBalanceUpError),
}