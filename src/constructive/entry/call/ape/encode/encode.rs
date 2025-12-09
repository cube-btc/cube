use crate::constructive::entry::call::ape::encode::error::encode_error::CallAPEEncodeError;
use crate::{
    constructive::{
        entry::call::call::Call,
        valtype::{val::atomic_val::atomic_val::AtomicVal, val::short_val::short_val::ShortVal},
    },
    inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER,
};
use bit_vec::BitVec;

impl Call {
    /// Airly Payload Encoding (APE) encoding for `Call`.
    ///
    /// This function encodes a `Call` as an Airly Payload Encoding (APE) bit vector.
    /// The `Call` can be a `Call` with a `AccountKey`, `ContractId`, `MethodIndex`, `Args`, `OpsBudget`, and `OpsPriceBase`.
    ///
    /// # Arguments
    /// * `&self` - The `Call` to encode.
    /// * `account_key` - The account key of the `Call`.
    /// * `registery_manager` - The registery manager to get the contract rank and body.
    /// * `ops_price_base` - The base ops price of the `Call`.
    pub async fn encode_ape(
        &self,
        account_key: [u8; 32],
        registery_manager: &REGISTERY_MANAGER,
        ops_price_base: u32,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
    ) -> Result<BitVec, CallAPEEncodeError> {
        // Initialize empty bit vector.
        let mut bits = BitVec::new();

        // Match the account key.
        if account_key != self.account_key {
            return Err(CallAPEEncodeError::AccountKeyMismatch(
                account_key,
                self.account_key,
            ));
        }

        // Contract rank
        let contract_rank = {
            let _registery_manager = registery_manager.lock().await;
            _registery_manager.get_rank_by_contract_id(self.contract_id)
        }
        .ok_or(CallAPEEncodeError::ContractRankNotFoundAtContractId(
            self.contract_id,
        ))?;

        // Contract rank as shortval
        let contract_rank_as_shortval = ShortVal::new(contract_rank as u32);

        // Extend the contract rank as shortval.
        bits.extend(contract_rank_as_shortval.encode_ape());

        // Methods length
        let contract_methods_count = {
            // Lock the registery manager.
            let _registery_manager = registery_manager.lock().await;

            // Get the contract body by contract id.
            let contract_body = _registery_manager
                .get_contract_body_by_contract_id(self.contract_id)
                .ok_or(CallAPEEncodeError::ContractBodyNotFoundAtContractId(
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
                .encode_ape()
                .map_err(|e| CallAPEEncodeError::MethodIndexAPEEncodeError(e))?,
        );

        // Extend the args.
        // No need to encode the args length.
        for arg in self.args.iter() {
            bits.extend(
                arg.encode_ape(
                    encode_account_rank_as_longval,
                    encode_contract_rank_as_longval,
                )
                .map_err(|e| CallAPEEncodeError::CallElementAPEEncodeError(e))?,
            );
        }

        // Ops budget as shortval
        let ops_budget_as_shortval = ShortVal::new(self.ops_budget as u32);

        // Extend the ops budget.
        bits.extend(ops_budget_as_shortval.encode_ape());

        // Match the ops price base.
        if ops_price_base != self.ops_price_base {
            return Err(CallAPEEncodeError::BaseOpsPriceMismatch(
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
                bits.extend(ops_price_extra_in_as_shortval.encode_ape());
            }
        }

        // Return the bits.
        Ok(bits)
    }
}
