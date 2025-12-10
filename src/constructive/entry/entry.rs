use crate::constructive::calldata::element::element::CalldataElement;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entry::entries::call::call::Call;
use serde::{Deserialize, Serialize};

/// Represents an `Entry`.
///
/// An `Entry` is a container for specific actions, such as calling a `Contract` or moving coins.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entry {
    //Move(MoveEntry),
    Call(Call),
    //Add(AddEntry),
    //Sub(SubEntry),
    //Liftup(LiftupEntry),
    //Swapout(SwapoutEntry),
    //Deploy(DeployEntry),
    //Config(ConfigEntry),
    //Nop(NopEntry),
    //Fail(FailEntry),
}

impl Entry {
    /// Returns the account key.
    pub fn new_call(
        account: Account,
        contract: Contract,
        method_index: u8,
        calldata_elements: Vec<CalldataElement>,
        ops_budget: Option<u32>,
        ops_price_base: u32,
        ops_price_overhead: Option<u32>,
    ) -> Self {
        Self::Call(Call::new(
            account,
            contract,
            method_index,
            calldata_elements,
            ops_budget,
            ops_price_base,
            ops_price_overhead,
        ))
    }
}
