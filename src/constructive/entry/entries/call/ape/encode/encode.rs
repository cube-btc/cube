use crate::constructive::entry::entries::call::ape::encode::error::encode_error::CallAPEEncodeError;
use crate::constructive::{
    entry::entries::call::call::Call,
    valtype::{val::atomic_val::atomic_val::AtomicVal, val::short_val::short_val::ShortVal},
};
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;

impl Call {
    /// Airly Payload Encoding (APE) encoding for `Call`.
    ///
    /// This function encodes a `Call` as an Airly Payload Encoding (APE) bit vector.
    /// The `Call` can be a `Call` with a `AccountKey`, `ContractId`, `MethodIndex`, `Args`, `OpsBudget`, and `OpsPriceBase`.
    ///
    /// # Arguments
    /// * `&self` - The `Call` to encode.
    /// * `registery_manager` - The guarded `RegisteryManager` to get the `Account`'s rank value.
    /// * `registery_manager` - The registery manager to get the contract rank and body.
    /// * `ops_price_base` - The base ops price of the `Call`.
    pub async fn encode_ape(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
    ) -> Result<BitVec, CallAPEEncodeError> {
        // 1 Initialize the `Call` APE bit vector.
        let mut bits = BitVec::new();

        // 2 Encode the `Account` into an APE bit vector.
        {
            // 2.1 Encode the `Account` into an APE bit vector.
            let account_bit_vector = self
                .account
                .encode_ape(registery_manager, encode_account_rank_as_longval)
                .await
                .map_err(|e| CallAPEEncodeError::AccountAPEEncodeError(e))?;

            // 2.2 Extend the `Entry` APE bit vector with the `Account` APE bit vector.
            bits.extend(account_bit_vector);
        }

        // 3 Encode the `Contract` into an APE bit vector.
        {
            // 3.1 Encode the `Contract` into an APE bit vector.
            let contract_bit_vector = self
                .contract
                .encode_ape(registery_manager, encode_contract_rank_as_longval)
                .await
                .map_err(|e| CallAPEEncodeError::ContractAPEEncodeError(e))?;

            // 3.2 Extend the `Call` APE bit vector with the `Contract` APE bit vector.
            bits.extend(contract_bit_vector);
        }

        // 3 Encode method call.
        {
            // 3.1 Get the `Contract` methods length.
            let contract_methods_count = self.contract.methods_len() as u8;

            // 3.2 Convert the method index to an `AtomicVal`.
            let method_call_as_atomicval =
                AtomicVal::new(self.method_index, contract_methods_count);

            // 3.3 Encode the `AtomicVal` into an APE bit vector.
            let method_call_bits = method_call_as_atomicval
                .encode_ape()
                .map_err(|e| CallAPEEncodeError::MethodCallAPEEncodeError(e))?;

            // 3.4 Extend the `Call` APE bit vector with the `AtomicVal` APE bit vector.
            bits.extend(method_call_bits);
        }

        // 4 Encode calldata.
        {
            // 4.1 Iterate over each calldata element.
            for calldata_element in self.calldata_elements.iter() {
                // 4.1.1 Encode the `CalldataElement` into an APE bit vector.
                let calldata_element_bits = calldata_element
                    .encode_ape(
                        registery_manager,
                        encode_account_rank_as_longval,
                        encode_contract_rank_as_longval,
                    )
                    .await
                    .map_err(|e| CallAPEEncodeError::CalldataElementAPEEncodeError(e))?;

                // 4.1.2 Extend the `Call` APE bit vector with the `CalldataElement` APE bit vector.
                bits.extend(calldata_element_bits);
            }
        }

        // 5 Encode ops budget.
        {
            // 5.1 Match the ops budget.
            match self.ops_budget {
                // 5.1.a The ops budget is present.
                Some(ops_budget) => {
                    // 5.1.a.1 Push true for this field being present.
                    bits.push(true);

                    // 5.1.a.2 Convert the ops budget to a shortval.
                    let ops_budget_as_shortval = ShortVal::new(ops_budget as u32);

                    // 5.1.a.3 Extend the ops budget.
                    bits.extend(ops_budget_as_shortval.encode_ape());
                }

                // 5.1.b The ops budget is absent.
                None => {
                    // 5.1.2 Push false for this field being absent.
                    bits.push(false);
                }
            }
        }

        // 6 Encode ops price.
        {
            // 6.1 Match on whether the ops price overhead is present.
            match self.ops_price_overhead {
                // 6.1.a The ops price has overhead.
                Some(ops_price_overhead) => {
                    // 6.1.a.1 Push true for this field being present.
                    bits.push(true);

                    // 6.1.a.2 Convert the ops price overhead to a shortval.
                    let ops_price_overhead_as_shortval = ShortVal::new(ops_price_overhead as u32);

                    // 6.1.a.3 Extend the ops price overhead.
                    bits.extend(ops_price_overhead_as_shortval.encode_ape());
                }

                // 6.1.b The ops price has no overhead.
                None => {
                    // 6.1.b.1 Push false for this field being absent.
                    bits.push(false);
                }
            }
        }

        // 7 Return the `Call` APE bit vector.
        Ok(bits)
    }
}
