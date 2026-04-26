use crate::inscriptive::archival_manager::errors::insert_error::ArchivalManagerInsertBatchRecordError;
use crate::inscriptive::coin_manager::errors::apply_changes_errors::CMApplyChangesError;
use crate::inscriptive::flame_manager::errors::apply_changes_error::FMApplyChangesError;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::registery::errors::apply_changes_error::RMApplyChangesError;
use crate::inscriptive::state_manager::errors::apply_changes_error::SMApplyChangesError;

/// Errors associated with applying changes to the `ExecCtx`.
#[derive(Debug, Clone)]
pub enum ApplyChangesError {
    CoinManagerApplyChangesError(CMApplyChangesError),
    GraveyardApplyChangesError(GraveyardApplyChangesError),
    RegisteryApplyChangesError(RMApplyChangesError),
    StateManagerApplyChangesError(SMApplyChangesError),
    PrivilegesManagerApplyChangesError(sled::Error),
    FlameManagerApplyChangesError(FMApplyChangesError),
    ArchivalManagerInsertBatchRecordError(ArchivalManagerInsertBatchRecordError),
}
