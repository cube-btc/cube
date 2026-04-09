/// The status of the flow encounter.
#[derive(Debug, Clone, PartialEq)]
pub enum FlowStatus {
    Active,
    Inactive,
    Uncovered,
}
