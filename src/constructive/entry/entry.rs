use crate::constructive::calldata::element::element::CallElement;
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
        args: Vec<CallElement>,
        ops_budget: u32,
        ops_price_base: u32,
        ops_price_extra_in: Option<u32>,
    ) -> Self {
        Self::Call(Call::new(
            account,
            contract,
            method_index,
            args,
            ops_budget,
            ops_price_base,
            ops_price_extra_in,
        ))
    }
}
