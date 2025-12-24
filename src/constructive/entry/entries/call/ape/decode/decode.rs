use crate::constructive::calldata::element::element::CalldataElement;
use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entry::entries::call::ape::decode::error::decode_error::CallEntryAPEDecodeError;
use crate::constructive::entry::entries::call::call::Call;
use crate::constructive::valtype::val::atomic_val::atomic_val::AtomicVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;

impl Call {
    /// Decodes a `Call` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes a `Call` as an Airly Payload Encoding (APE) bit vector.
    /// The `Call` can be a `Call` with a `Account`, `Contract`, `MethodIndex`, `Args`, `OpsBudget`, and `OpsPriceBase`.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `base_ops_price` - The base ops price of the `Call`.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_account_rank_as_longval` - Whether to decode the account rank as a `LongVal` or a `ShortVal`.
    /// * `decode_contract_rank_as_longval` - Whether to decode the contract rank as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
        base_ops_price: u32,
        registery_manager: &REGISTERY_MANAGER,
        decode_account_rank_as_longval: bool,
        decode_contract_rank_as_longval: bool,
    ) -> Result<Call, CallEntryAPEDecodeError> {
        // 1 Decode the `RootAccount` from the APE bitstream.
        let account: RootAccount = RootAccount::decode_ape(
            bit_stream,
            registery_manager,
            decode_account_rank_as_longval,
        )
        .await
        .map_err(|e| CallEntryAPEDecodeError::AccountAPEDecodeError(e))?;

        // 2 Decode the `Contract` from the APE bitstream.
        let contract: Contract = Contract::decode_ape(
            bit_stream,
            registery_manager,
            decode_contract_rank_as_longval,
        )
        .await
        .map_err(|e| CallEntryAPEDecodeError::ContractAPEDecodeError(e))?;

        // 3 Decode the `Method Call` from the APE bitstream.
        let method_call: u8 = AtomicVal::decode_ape(bit_stream, contract.methods_len() as u8)
            .map_err(|e| CallEntryAPEDecodeError::MethodIndexAPEDecodeError(e))?
            .value()
            .into();

        // 4 Decode the `Calldata Elements` from the APE bitstream.
        let calldata_elements: Vec<CalldataElement> = {
            // 4.1 Initialize a vector to store the `Calldata Elements`.
            let mut calldata_elements: Vec<CalldataElement> = Vec::new();

            // 4.2 Iterate over each `Calldata Element`.
            for _ in 0..contract.methods_len() {
                // 4.2.1 Decode the `Calldata Element` from the APE bitstream.
                let calldata_element: CalldataElement = CalldataElement::decode_ape(
                    bit_stream,
                    CalldataElementType::U8,
                    registery_manager,
                    false,
                )
                .await
                .map_err(|e| CallEntryAPEDecodeError::CalldataElementAPEDecodeError(e))?;

                // 4.2.2 Push the `Calldata Element` to the vector.
                calldata_elements.push(calldata_element);
            }

            // 4.3 Return the vector of `Calldata Elements`.
            calldata_elements
        };

        // 5 Collect one bit to determine if the `Ops Budget` is present.
        let ops_budget_present: bool = bit_stream
            .next()
            .ok_or(CallEntryAPEDecodeError::OpsBudgetPresentBitCollectError)?;

        // 6 Match on the `Ops Budget` presence.
        let ops_budget: Option<u32> = match ops_budget_present {
            // 6.a The `Ops Budget` is present.
            true => {
                // 6.a.1 Decode the `Ops Budget` from the APE bitstream as a `ShortVal`.
                let ops_budget: u32 = ShortVal::decode_ape(bit_stream)
                    .map_err(|e| CallEntryAPEDecodeError::OpsBudgetAPEDecodeError(e))?
                    .value()
                    .into();

                // 6.a.2 Return the `Ops Budget`.
                Some(ops_budget)
            }

            // 6.b The `Ops Budget` is absent.
            false => None,
        };
        // 7 Collect one bit to determine if the `Ops Price Overhead` is present.
        let ops_price_overhead_present: bool = bit_stream
            .next()
            .ok_or(CallEntryAPEDecodeError::OpsPriceOverheadPresentBitCollectError)?;

        // 8 Match on the `Ops Price Overhead` presence.
        let ops_price_overhead: Option<u32> = match ops_price_overhead_present {
            // 8.a The `Ops Price Overhead` is present.
            true => Some(
                ShortVal::decode_ape(bit_stream)
                    .map_err(|e| CallEntryAPEDecodeError::OpsPriceOverheadAPEDecodeError(e))?
                    .value()
                    .into(),
            ),
            // 8.b The `Ops Price Overhead` is absent.
            false => None,
        };

        // 9 Construct the `Call`.
        let call = Call::new(
            account,
            contract,
            method_call,
            calldata_elements,
            ops_budget,
            base_ops_price,
            ops_price_overhead,
        );

        // 10 Return the `Call`.
        Ok(call)
    }
}
