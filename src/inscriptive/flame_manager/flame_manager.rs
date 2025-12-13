use crate::inscriptive::flame_manager::flame::flame::Flame;
use crate::inscriptive::flame_manager::flame_config::flame_config::FlameConfig;
use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Rollup height.
type RollupHeight = u64;

/// Flame manager.
#[allow(dead_code)]
pub struct FlameManager {
    // Account flame configs
    account_flame_configs: HashMap<AccountKey, FlameConfig>,

    // Projected flames
    projected_flames: HashMap<RollupHeight, Vec<Flame>>,

    // Account flame sets
    account_flame_sets: HashMap<AccountKey, Vec<Flame>>,
}
