use crate::constructive::core_types::calldata::calldata_elements::validation::CalldataElementValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementSBEEncodeError {
    ValidationError(CalldataElementValidationError),
    ElementCountTooLargeForU32 { len: usize },
}
