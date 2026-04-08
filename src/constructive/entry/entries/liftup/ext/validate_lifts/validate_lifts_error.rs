/// Errors associated with validating the `Lift`s in a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupValidateLiftsError {
    InvalidLiftStructureError,
    InvalidLiftUTXOError,
}
