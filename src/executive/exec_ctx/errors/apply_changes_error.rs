use crate::inscriptive::coin_manager::errors::apply_changes_errors::CMApplyChangesError;
use crate::inscriptive::flame_manager::errors::apply_changes_error::FMApplyChangesError;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::registery::errors::apply_changes_error::RMApplyChangesError;

/// Errors associated with applying changes to the `ExecCtx`.
#[derive(Debug, Clone)]
pub enum ApplyChangesError {
    CoinManagerApplyChangesError(CMApplyChangesError),
    GraveyardApplyChangesError(GraveyardApplyChangesError),
    RegisteryApplyChangesError(RMApplyChangesError),
    FlameManagerApplyChangesError(FMApplyChangesError),
}
