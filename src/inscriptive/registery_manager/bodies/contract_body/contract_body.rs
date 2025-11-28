use crate::executive::program::program::Program;

/// A struct for containing the registery index and call counter of a contract.
#[derive(Clone)]
pub struct RMContractBody {
    // Assigned registery index of a deployed contract.
    pub registery_index: u32,

    // Ever-increasing call counter of a contract.
    pub call_counter: u64,

    // Decompiled program of a contract.
    pub program: Program,
}

impl RMContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(registery_index: u32, call_counter: u64, program: Program) -> Self {
        Self {
            registery_index,
            call_counter,
            program,
        }
    }
}
