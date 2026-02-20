/// A struct for containing epheremal state differences to be applied for `PrivilegesManager`.
pub struct PrivilegesManagerDelta {}

impl PrivilegesManagerDelta {
    /// Creates a fresh new delta.
    pub fn fresh_new() -> Self {
        Self {}
    }
}