use crate::constructive::core_types::calldata::calldata_elements::calldata_element::CalldataElement;
use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entry::entry_kinds::call::ext::codec::ape::decode::error::decode_error::CallEntryAPEDecodeError;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::registery::REGISTERY;

impl Call {
    /// Decodes a `Call` from an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        execution_batch_height: u64,
        base_ops_price: u32,
        decode_account_rank_as_longval: bool,
        decode_contract_rank_as_longval: bool,
        registery: &REGISTERY,
    ) -> Result<Call, CallEntryAPEDecodeError> {
        let account: RootAccount = RootAccount::decode_ape(
            bit_stream,
            decode_account_rank_as_longval,
            registery,
        )
        .await
        .map_err(CallEntryAPEDecodeError::AccountAPEDecodeError)?;

        let contract: Contract =
            Contract::decode_ape(bit_stream, registery, decode_contract_rank_as_longval)
                .await
                .map_err(CallEntryAPEDecodeError::ContractAPEDecodeError)?;

        let methods_len = {
            let _registery = registery.lock().await;
            _registery
                .get_contract_methods_len_by_contract_id(contract.contract_id())
                .ok_or(
                    CallEntryAPEDecodeError::UnableToRetrieveContractMethodsLenFromRegistery(
                        contract.contract_id(),
                    ),
                )?
        };

        let method_index = MethodIndex::decode_ape(bit_stream, methods_len)
            .map_err(CallEntryAPEDecodeError::MethodIndexAPEDecodeError)?;

        let calldata_count = ShortVal::decode_ape(bit_stream)
            .map_err(CallEntryAPEDecodeError::CalldataCountAPEDecodeError)?
            .value() as usize;

        let calldata_elements: Vec<CalldataElement> = {
            let mut calldata_elements: Vec<CalldataElement> = Vec::new();

            for _ in 0..calldata_count {
                let calldata_element: CalldataElement = CalldataElement::decode_ape(
                    bit_stream,
                    CalldataElementType::U8,
                    registery,
                    false,
                )
                .await
                .map_err(CallEntryAPEDecodeError::CalldataElementAPEDecodeError)?;

                calldata_elements.push(calldata_element);
            }

            calldata_elements
        };

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
