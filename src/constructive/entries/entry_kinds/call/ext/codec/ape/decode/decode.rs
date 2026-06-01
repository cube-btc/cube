use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::core_types::calldata::calldata_elements::calldata_element::CalldataElement;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::calldata::element::ape::decode::error::decode_errors::CalldataElementAPEDecodeError;
use crate::constructive::entry::entry_kinds::call::ext::codec::ape::decode::error::decode_error::CallEntryAPEDecodeError;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registry::registry::REGISTRY;

impl Call {
    /// Decodes a `Call` from an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        execution_batch_height: u64,
        base_ops_price: u32,
        decode_account_rank_as_longval: bool,
        decode_contract_rank_as_longval: bool,
        registry: &REGISTRY,
    ) -> Result<Call, CallEntryAPEDecodeError> {
        let account: RootAccount = RootAccount::decode_ape(
            bit_stream,
            decode_account_rank_as_longval,
            registry,
        )
        .await
        .map_err(CallEntryAPEDecodeError::AccountAPEDecodeError)?;

        let contract: Contract =
            Contract::decode_ape(bit_stream, registry, decode_contract_rank_as_longval)
                .await
                .map_err(CallEntryAPEDecodeError::ContractAPEDecodeError)?;

        let contract_id = contract.contract_id();

        let methods_len = {
            let _registry = registry.lock().await;
            _registry
                .get_contract_methods_len_by_contract_id(contract_id)
                .ok_or(
                    CallEntryAPEDecodeError::UnableToRetrieveContractMethodsLenFromRegistry(
                        contract_id,
                    ),
                )?
        };

        let method_index = MethodIndex::decode_ape(bit_stream, methods_len)
            .map_err(CallEntryAPEDecodeError::MethodIndexAPEDecodeError)?;

        let arg_types = {
            let _registry = registry.lock().await;
            _registry
                .get_contract_method_arg_types_by_contract_id_and_method_index(
                    contract_id,
                    method_index.index(),
                )
                .ok_or(CallEntryAPEDecodeError::UnableToRetrieveMethodArgTypesFromRegistry {
                    contract_id,
                    method_index: method_index.index(),
                })?
        };

        let calldata_count = ShortVal::decode_ape(bit_stream)
            .map_err(CallEntryAPEDecodeError::CalldataCountAPEDecodeError)?
            .value() as usize;

        if calldata_count != arg_types.len() {
            return Err(CallEntryAPEDecodeError::CalldataCountMismatch {
                expected: arg_types.len(),
                got: calldata_count,
            });
        }

        let mut calldata_elements = Vec::with_capacity(calldata_count);
        for arg_type in arg_types {
            let decode_rank_as_longval = match arg_type {
                CalldataElementType::Account => decode_account_rank_as_longval,
                CalldataElementType::Contract => decode_contract_rank_as_longval,
                _ => false,
            };

            let calldata_element = CalldataElement::decode_ape(
                bit_stream,
                arg_type,
                registry,
                decode_rank_as_longval,
            )
            .await
            .map_err(CallEntryAPEDecodeError::CalldataElementAPEDecodeError)?;

            calldata_element
                .validate()
                .map_err(|e| {
                    CallEntryAPEDecodeError::CalldataElementAPEDecodeError(
                        CalldataElementAPEDecodeError::ValidationError(e),
                    )
                })?;

            calldata_elements.push(calldata_element);
        }

        let ops_budget = OpsBudget::decode_ape(bit_stream)
            .map_err(CallEntryAPEDecodeError::OpsBudgetAPEDecodeError)?;

        let ops_price = OpsPrice::decode_ape(bit_stream, base_ops_price)
            .map_err(CallEntryAPEDecodeError::OpsPriceAPEDecodeError)?;

        let target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(CallEntryAPEDecodeError::TargetAPEDecodeError)?;

        Ok(Call::new(
            account,
            contract,
            method_index,
            calldata_elements,
            ops_budget,
            ops_price,
            target,
        ))
    }
}
