/// Enum to represent errors that can occur when encoding an `AtomicVal` into a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomicValAPEEncodeError {
    U8ExtToBitsError,
}
