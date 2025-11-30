use super::flame_config::flame_config::FlameConfig;

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type SecondaryAggregationKey = Vec<u8>;

// A struct for containing the registery index and call counter of an account.
#[derive(Clone)]
pub struct RMAccountBody {
    // Assigned registery index of an account.
    pub registery_index: u32,

    // Ever-increasing call counter of an account.
    pub call_counter: u64,

    // BLS key of an account.
    pub primary_bls_key: AccountBLSKey,

    // Secondary aggregation key of an account.
    pub secondary_aggregation_key: Option<SecondaryAggregationKey>,

    // Flame config of an account.
    pub flame_config: FlameConfig,
}

impl RMAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(
        registery_index: u32,
        call_counter: u64,
        primary_bls_key: AccountBLSKey,
        secondary_aggregation_key: Option<SecondaryAggregationKey>,
        flame_config: FlameConfig,
    ) -> Self {
        Self {
            registery_index,
            call_counter,
            primary_bls_key,
            secondary_aggregation_key,
            flame_config,
        }
    }
}
