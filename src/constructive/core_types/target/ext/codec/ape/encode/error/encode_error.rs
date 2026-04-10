/// Enum to represent errors that can occur when encoding a `Target` into an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetAPEEncodeError {
    TargetAfterExecution,
    TargetTooFarInPast,
}
