use crate::inscriptive::coin_manager::errors::apply_changes_errors::CMApplyChangesError;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::registery::errors::apply_changes_error::RMApplyChangesError;

/// Errors associated with applying changes to the `ExecContainer`.
#[derive(Debug, Clone)]
pub enum ApplyChangesError {
    CoinManagerApplyChangesError(CMApplyChangesError),
    GraveyardApplyChangesError(GraveyardApplyChangesError),
    RegisteryApplyChangesError(RMApplyChangesError),
}
