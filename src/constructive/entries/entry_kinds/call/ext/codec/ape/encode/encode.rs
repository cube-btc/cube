use crate::constructive::entry::entry_kinds::call::ext::codec::ape::encode::error::encode_error::CallAPEEncodeError;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registry::registry::REGISTRY;
use bit_vec::BitVec;

impl Call {
    /// Airly Payload Encoding (APE) encoding for `Call`.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registry: &REGISTRY,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
        base_ops_price: u32,
    ) -> Result<BitVec, CallAPEEncodeError> {
        let mut bits = BitVec::new();

        let account_bit_vector = self
            .account
            .encode_ape(registry, encode_account_rank_as_longval)
            .await
            .map_err(CallAPEEncodeError::AccountAPEEncodeError)?;
        bits.extend(account_bit_vector);

        let contract_bit_vector = self
            .contract
            .encode_ape(registry, encode_contract_rank_as_longval)
            .await
            .map_err(CallAPEEncodeError::ContractAPEEncodeError)?;
        bits.extend(contract_bit_vector);

        let contract_id = self.contract.contract_id();

        let methods_len = {
            let _registry = registry.lock().await;
            _registry
                .get_contract_methods_len_by_contract_id(contract_id)
                .ok_or(CallAPEEncodeError::UnableToRetrieveContractMethodsLenFromRegistry(
                    contract_id,
                ))?
        };

        bits.extend(
            self.method_index
                .encode_ape(methods_len)
                .map_err(CallAPEEncodeError::MethodIndexAPEEncodeError)?,
        );

        let arg_types = {
            let _registry = registry.lock().await;
            _registry
                .get_contract_method_arg_types_by_contract_id_and_method_index(
                    contract_id,
                    self.method_index.index(),
                )
                .ok_or(CallAPEEncodeError::UnableToRetrieveMethodArgTypesFromRegistry {
                    contract_id,
                    method_index: self.method_index.index(),
                })?
        };

        if self.calldata_elements.len() != arg_types.len() {
            return Err(CallAPEEncodeError::CalldataCountMismatch {
                expected: arg_types.len(),
                got: self.calldata_elements.len(),
            });
        }

        bits.extend(
            ShortVal::new(self.calldata_elements.len() as u32).encode_ape(),
        );

        for calldata_element in self.calldata_elements.iter() {
            calldata_element
                .validate()
                .map_err(CallAPEEncodeError::CalldataElementValidationError)?;

            let calldata_element_bits = calldata_element
                .encode_ape(
                    registry,
                    encode_account_rank_as_longval,
                    encode_contract_rank_as_longval,
                )
                .await
                .map_err(CallAPEEncodeError::CalldataElementAPEEncodeError)?;
            bits.extend(calldata_element_bits);
        }

        bits.extend(self.ops_budget.encode_ape());

        let ops_price_bits = self
            .ops_price
            .encode_ape(base_ops_price)
            .map_err(CallAPEEncodeError::OpsPriceAPEEncodeError)?;
        bits.extend(ops_price_bits);

        let target_bits = self
            .target
            .encode_ape(execution_batch_height)
            .map_err(CallAPEEncodeError::TargetAPEEncodeError)?;
        bits.extend(target_bits);

        Ok(bits)
    }
}
