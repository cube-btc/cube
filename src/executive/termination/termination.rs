use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::executive::termination::error::termination_error::TerminationError;

impl ExecCtx {
    /// Executes a periodic termination.
    pub async fn periodic_termination(
        &mut self,
        _execution_timestamp: u64,
    ) -> Result<(), TerminationError> {
        // 1. Accounts phase
        {
            // 1.1 Sets Plebs 'LivenessFlag' to 'ToBeDestroyed' if it hasnt been inactive for some period..
        }

        // 2. Contracts phase
        {}

        Ok(())
    }
}
