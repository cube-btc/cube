use crate::constructive::entry::entry::Entry;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::{
    constructive::entry::entries::liftup::liftup::Liftup,
    executive::exec_container::errors::apply_changes_error::ApplyChangesError,
    executive::exec_container::errors::liftup_execution_error::LiftupExecutionError,
};
use bitcoin::{OutPoint, TxOut};
use std::sync::Arc;
use tokio::sync::Mutex;

/// `ExecContainer` contains a set of executed entries.
pub struct ExecContainer {
    // The key of the Engine.
    pub engine_key: [u8; 32],

    // The local registery database of the Engine.
    pub registery: REGISTERY,

    // The local utxo set database of the Engine.
    pub utxo_set: UTXO_SET,

    // The local graveyard database of the Engine.
    pub graveyard: GRAVEYARD,

    // The local coin manager database of the Engine.
    pub coin_manager: COIN_MANAGER,

    // The local flame manager database of the Engine.
    pub flame_manager: FLAME_MANAGER,

    // The entries that have been executed in the pool.
    pub executed_entries: Vec<Entry>,

    // The Bitcoin transaction inputs that have been added as a result of executing `Liftup` entries.
    pub added_tx_inputs: Vec<(OutPoint, TxOut)>,

    // The Bitcoin transaction outputs that have been added as a result of executing `Swapout` entries.
    pub added_tx_outputs: Vec<TxOut>,
}

/// Guarded `ExecContainer`.
#[allow(non_camel_case_types)]
pub type EXEC_CONTAINER = Arc<Mutex<ExecContainer>>;

impl ExecContainer {
    /// Constructs the `ExecContainer`.    
    pub fn construct(
        engine_key: [u8; 32],
        registery: REGISTERY,
        utxo_set: UTXO_SET,
        graveyard: GRAVEYARD,
        coin_manager: COIN_MANAGER,
        flame_manager: FLAME_MANAGER,
    ) -> EXEC_CONTAINER {
        // 1 Initialize the `ExecContainer`.
        let exec_container = ExecContainer {
            engine_key,
            registery,
            utxo_set,
            graveyard,
            coin_manager,
            flame_manager,
            executed_entries: Vec::new(),
            added_tx_inputs: Vec::new(),
            added_tx_outputs: Vec::new(),
        };

        // 2 Return the guarded `ExecContainer`.
        Arc::new(Mutex::new(exec_container))
    }

    /// Prepares the `ExecContainer` prior to each execution.
    async fn pre_execution(&mut self) {
        // 1 Pre-execution coin manager.
        self.coin_manager.lock().await.pre_execution();

        // 2 Pre-execution graveyard.
        self.graveyard.lock().await.pre_execution();

        // 3 Pre-execution registery.
        self.registery.lock().await.pre_execution();

        // 4 Pre-execution flame manager.
        self.flame_manager.lock().await.pre_execution();
    }

    /// Rolls back the last execution of the `ExecContainer` due to a failed individual Entry execution.
    async fn rollback_last(&mut self) {
        // 1 Rollback last coin manager.
        self.coin_manager.lock().await.rollback_last();

        // 2 Rollback last graveyard.
        self.graveyard.lock().await.rollback_last();

        // 3 Rollback last registery.
        self.registery.lock().await.rollback_last();

        // 4 Rollback last flame manager.
        self.flame_manager.lock().await.rollback_last();
    }

    /// Applies the changes to the `ExecContainer` collectively for all Entries in the container.
    async fn apply_changes(
        &mut self,
        new_projector_height: u64,
        projector_expiry_height: u64,
    ) -> Result<(), ApplyChangesError> {
        // 1 Apply changes to the coin manager.
        if let Err(error) = self.coin_manager.lock().await.apply_changes() {
            return Err(ApplyChangesError::CoinManagerApplyChangesError(error));
        }

        // 2 Apply changes to the graveyard.
        if let Err(error) = self.graveyard.lock().await.apply_changes() {
            return Err(ApplyChangesError::GraveyardApplyChangesError(error));
        }

        // 3 Apply changes to the registery.
        if let Err(error) = self.registery.lock().await.apply_changes() {
            return Err(ApplyChangesError::RegisteryApplyChangesError(error));
        }

        // 4 Apply changes to the flame manager.
        if let Err(error) = self
            .flame_manager
            .lock()
            .await
            .apply_changes(
                &self.coin_manager,
                &self.registery,
                new_projector_height,
                projector_expiry_height,
            )
            .await
        {
            return Err(ApplyChangesError::FlameManagerApplyChangesError(error));
        }

        // 5 Flush the container.
        {
            // 5.1 Clear the executed entries.
            self.executed_entries.clear();

            // 5.2 Clear the added Bitcoin transaction inputs.
            self.added_tx_inputs.clear();

            // 5.3 Clear the added Bitcoin transaction outputs.
            self.added_tx_outputs.clear();
        }

        // 5 Return Ok.
        Ok(())
    }

    /// Executes a `Liftup` entry in the pool.
    pub async fn execute_liftup(
        &mut self,
        liftup: Liftup,
        session_timestamp: u64,
        optimized: bool,
    ) -> Result<(), LiftupExecutionError> {
        // 1 Pre-execution.
        self.pre_execution().await;

        // 2 Execute the liftup.
        match self
            .execute_liftup_internal(&liftup, session_timestamp, optimized)
            .await
        {
            // 2.a Success.
            Ok(_) => {
                // 2.a.1 Add Lifts inside the Liftup to the added tx inputs.
                self.added_tx_inputs.extend(
                    liftup
                        .lift_prevtxos
                        .iter()
                        .map(|lift| (lift.outpoint(), lift.txout().clone())),
                );

                // 2.a.2 Construct the Liftup entry.
                let liftup_entry = Entry::new_liftup(liftup);

                // 2.a.3 Add the liftup entry to the executed entries.
                self.executed_entries.push(liftup_entry);

                // 2.a.4 Return Ok.
                Ok(())
            }
            // 2.b Error.
            Err(error) => {
                // 2.b.1 Rollback last.
                self.rollback_last().await;

                // 2.b.2 Return the error.
                Err(error)
            }
        }
    }
}
