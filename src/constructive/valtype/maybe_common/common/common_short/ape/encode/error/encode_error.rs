/// Enum to represent errors that can occur when encoding a `CommonShortVal` into a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommonShortValAPEEncodeError {
    U8ExtToBitsError,
}
