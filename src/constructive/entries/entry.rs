use crate::constructive::entry::entry_types::call::call::Call;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
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
    Liftup(Liftup),
    //Swapout(SwapoutEntry),
    //Deploy(DeployEntry),
    //Config(ConfigEntry),
    //Nop(NopEntry),
    //Fail(FailEntry),
}

impl Entry {
    /// Creates a new call entry.
    pub fn new_call(call: Call) -> Self {
        Self::Call(call)
    }

    /// Creates a new liftup entry.
    pub fn new_liftup(liftup: Liftup) -> Self {
        Self::Liftup(liftup)
    }
}
