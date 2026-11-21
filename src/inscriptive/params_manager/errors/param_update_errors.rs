/// Errors associated with updating a configured pleb inactivity termination period.
#[derive(Debug, Clone)]
pub enum PMConfiguredPlebInactivityTerminationPeriodUpdateError {
    BelowFloor(u64, u64),
}

/// Errors associated with updating an unconfigured pleb inactivity termination period.
#[derive(Debug, Clone)]
pub enum PMUnconfiguredPlebInactivityTerminationPeriodUpdateError {
    BelowFloor(u64, u64),
}

/// Errors associated with updating a configured pleb ToBeDestroyed recovery gap.
#[derive(Debug, Clone)]
pub enum PMConfiguredPlebToBeDestroyedRecoveryGapUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}

/// Errors associated with updating an unconfigured pleb ToBeDestroyed recovery gap.
#[derive(Debug, Clone)]
pub enum PMUnconfiguredPlebToBeDestroyedRecoveryGapUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}

/// Errors associated with updating a mutable contract vitality sucked termination value.
///
/// Valid range is
/// `MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_CEILING..=MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_FLOOR`
/// (most-negative ..= least-negative).
#[derive(Debug, Clone)]
pub enum PMMutableContractVitalitySuckedTerminationValueUpdateError {
    BelowCeiling(i64, i64),
    AboveFloor(i64, i64),
}

/// Errors associated with updating a contract statesize PPM tax rate.
#[derive(Debug, Clone)]
pub enum PMContractStatesizePpmTaxRateUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}

/// Errors associated with updating a contract binsize PPM tax rate.
#[derive(Debug, Clone)]
pub enum PMContractBinsizePpmTaxRateUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}

/// Errors associated with updating a contract shadowed liquidity PPM tax rate.
#[derive(Debug, Clone)]
pub enum PMContractShadowedLiquidityPpmTaxRateUpdateError {
    AboveCeiling(u64, u64),
}

/// Errors associated with updating a contract non-shadowed liquidity PPM tax rate.
#[derive(Debug, Clone)]
pub enum PMContractNonShadowedLiquidityPpmTaxRateUpdateError {
    AboveCeiling(u64, u64),
}

/// Errors associated with updating an account termination check period.
#[derive(Debug, Clone)]
pub enum PMAccountTerminationCheckPeriodUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}

/// Errors associated with updating a contract termination check period.
#[derive(Debug, Clone)]
pub enum PMContractTerminationCheckPeriodUpdateError {
    BelowFloor(u64, u64),
    AboveCeiling(u64, u64),
}
