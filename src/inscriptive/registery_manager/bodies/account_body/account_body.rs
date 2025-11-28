// A struct for containing the registery index and call counter of an account.
#[derive(Clone)]
pub struct RMAccountBody {
    // Assigned registery index of an account.
    pub registery_index: u32,

    // Ever-increasing call counter of an account.
    pub call_counter: u64,
}

impl RMAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(registery_index: u32, call_counter: u64) -> Self {
        Self {
            registery_index,
            call_counter,
        }
    }
}
