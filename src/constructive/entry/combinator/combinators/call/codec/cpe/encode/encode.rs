use crate::{
    constructive::{
        entry::combinator::combinators::call::{
            call::Call, codec::cpe::encode::encode_error::CallCPEEncodeError,
        },
        valtype::{val::atomic_val::atomic_val::AtomicVal, val::short_val::short_val::ShortVal},
    },
    inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER,
};
use bit_vec::BitVec;

impl Call {
    /// Encodes the call as a bit vector.
    pub async fn encode_cpe(
        &self,
        account_key: [u8; 32],
        registery_manager: &REGISTERY_MANAGER,
        ops_price_base: u32,
    ) -> Result<BitVec, CallCPEEncodeError> {
        // Initialize empty bit vector.
        let mut bits = BitVec::new();

        // Match the account key.
        if account_key != self.account_key {
            return Err(CallCPEEncodeError::AccountKeyMismatch(
                account_key,
                self.account_key,
            ));
        }

        // Contract rank
        let contract_rank = {
            let _registery_manager = registery_manager.lock().await;
            _registery_manager.get_rank_by_contract_id(self.contract_id)
        }
        .ok_or(CallCPEEncodeError::ContractRankNotFoundAtContractId(
            self.contract_id,
        ))?;

        // Contract rank as shortval
        let contract_rank_as_shortval = ShortVal::new(contract_rank as u32);

        // Extend the contract rank as shortval.
        bits.extend(contract_rank_as_shortval.encode_cpe());

        // Methods length
        let contract_methods_count = {
            // Lock the registery manager.
            let _registery_manager = registery_manager.lock().await;

            // Get the contract body by contract id.
            let contract_body = _registery_manager
                .get_contract_body_by_contract_id(self.contract_id)
                .ok_or(CallCPEEncodeError::ContractBodyNotFoundAtContractId(
                    self.contract_id,
                ))?;

            // Get the methods length.
            contract_body.executable.methods_len() as u8
        };

        // Method index as atomic value
        let method_index_as_atomicval = AtomicVal::new(self.method_index, contract_methods_count);

        // Extend the method index.
        bits.extend(
            method_index_as_atomicval
                .encode_cpe()
                .map_err(|e| CallCPEEncodeError::MethodIndexCPEEncodeError(e))?,
        );

        // Extend the args.
        // No need to encode the args length.
        for arg in self.args.iter() {
            bits.extend(arg.encode_cpe());
        }

        // Ops budget as shortval
        let ops_budget_as_shortval = ShortVal::new(self.ops_budget as u32);

        // Extend the ops budget.
        bits.extend(ops_budget_as_shortval.encode_cpe());

        // Match the ops price base.
        if ops_price_base != self.ops_price_base {
            return Err(CallCPEEncodeError::BaseOpsPriceMismatch(
                ops_price_base,
                self.ops_price_base,
            ));
        }

        // Match ops price extra in.
        match self.ops_price_extra_in {
            None => {
                // Push false for this field being absent.
                bits.push(false);
            }
            Some(ops_price_extra_in) => {
                // Push true for this field being present.
                bits.push(true);

                // Convert the ops price extra in to a shortval.
                let ops_price_extra_in_as_shortval = ShortVal::new(ops_price_extra_in as u32);

                // Extend the ops price extra in.
                bits.extend(ops_price_extra_in_as_shortval.encode_cpe());
            }
        }

        // Return the bits.
        Ok(bits)
    }
}
