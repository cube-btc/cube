use crate::constructive::entry::entry_kinds::liftup::ext::codec::sbe::encode::error::LiftupSBEEncodeError;

/// Errors that can occur when encoding an `Entry` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntrySBEEncodeError {
   
   LiftupSBEEncodeError(LiftupSBEEncodeError),

}
