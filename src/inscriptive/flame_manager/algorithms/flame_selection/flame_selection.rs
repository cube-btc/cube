use crate::inscriptive::flame_manager::flame::{flame::Flame, flame_tier::flame_tier::FlameTier};
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;

/// Satoshi amount.
type SatoshiAmount = u64;

/// Flame script pubkey.
type FlameScriptPubKey = Vec<u8>;

/// Returns the flames to fund from a given target value and current flame set sum value.
pub fn return_flames_to_fund(
    // Flame config.
    flame_config: &FMAccountFlameConfig,

    // Target value in satoshis.
    target_value: SatoshiAmount,

    // Current flame set sum value in satoshis.
    current_flame_set_sum_value: SatoshiAmount,
) -> Option<Vec<Flame>> {
    // 1 Check if flame config is ready.
    if !flame_config.is_ready() {
        return None;
    }

    // 2 Calculate the gap amount.
    let gap_amount: SatoshiAmount = match target_value.checked_sub(current_flame_set_sum_value) {
        // 2.1 Gap amount is none. Return none.
        None => return None,

        // 2.2 Gap amount is some. Return the gap amount.
        Some(gap_amount) => gap_amount,
    };

    // 3 Initialize the list of flames to return.
    let mut flames = Vec::<Flame>::new();

    // 4 Match on the ZKTLC tier any amount.
    match &flame_config.tier_any_amount {
        // 4.a Any amount tier is set.
        Some(tier_config) => {
            // 4.a.1 Construct the flame tier.
            let flame_tier = FlameTier::TierAnyAmount(gap_amount);

            // 4.a.2 Construct the flame.
            let flame = Flame::new(flame_tier, tier_config.script_pubkey.clone());

            // 4.a.3 Push the flame to the list of flames to return.
            flames.push(flame);
        }
        // 4.b Any amount tier is not set. We have to push a set of flames.
        None => {
            // 4.b.1 Collect available tiers with their values and script pubkeys.
            // We'll store them as (tier_value, script_pubkey, tier_enum) tuples.
            let mut available_tiers: Vec<(SatoshiAmount, FlameScriptPubKey, FlameTier)> =
                Vec::new();

            // 4.b.1.1 Check tier 7 (hundred million satoshis = 100000000).
            if let Some(tier_config) = &flame_config.tier_7_hundred_million_satoshis {
                available_tiers.push((
                    100000000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier7HundredMillionSatoshis,
                ));
            }

            // 4.b.1.2 Check tier 6 (ten million satoshis = 10000000).
            if let Some(tier_config) = &flame_config.tier_6_ten_million_satoshis {
                available_tiers.push((
                    10000000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier6TenMillionSatoshis,
                ));
            }

            // 4.b.1.3 Check tier 5 (one million satoshis = 1000000).
            if let Some(tier_config) = &flame_config.tier_5_one_million_satoshis {
                available_tiers.push((
                    1000000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier5HundredThousandSatoshis,
                ));
            }

            // 4.b.1.4 Check tier 4 (hundred thousand satoshis = 100000).
            if let Some(tier_config) = &flame_config.tier_4_hundred_thousand_satoshis {
                available_tiers.push((
                    100000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier4TenThousandSatoshis,
                ));
            }

            // 4.b.1.5 Check tier 3 (ten thousand satoshis = 10000).
            if let Some(tier_config) = &flame_config.tier_3_ten_thousand_satoshis {
                available_tiers.push((
                    10000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier3ThousandSatoshis,
                ));
            }

            // 4.b.1.6 Check tier 2 (thousand satoshis = 1000).
            if let Some(tier_config) = &flame_config.tier_2_thousand_satoshis {
                available_tiers.push((
                    1000,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier2ThousandSatoshis,
                ));
            }

            // 4.b.1.7 Check tier 1 (hundred satoshis = 100).
            if let Some(tier_config) = &flame_config.tier_1_hundred_satoshis {
                available_tiers.push((
                    100,
                    tier_config.script_pubkey.clone(),
                    FlameTier::Tier1HundredSatoshis,
                ));
            }

            // 4.b.1.8 If no tiers are available, return None.
            if available_tiers.is_empty() {
                return None;
            }

            // 4.b.2 Sort tiers in descending order by value (largest first).
            available_tiers.sort_by(|a, b| b.0.cmp(&a.0));

            // 4.b.2.1 Store the smallest tier for rounding up if needed.
            let smallest_tier = available_tiers.last().cloned();

            // 4.b.3 Greedily select tiers to round up the delta.
            let mut remaining_gap_amount = gap_amount;
            for (tier_value, script_pubkey, flame_tier) in &available_tiers {
                // 4.b.3.1 Calculate how many of this tier we need.
                let count = remaining_gap_amount / tier_value;

                // 4.b.3.2 If we need at least one of this tier, add them.
                if count > 0 {
                    for _ in 0..count {
                        let flame = Flame::new(*flame_tier, script_pubkey.clone());
                        flames.push(flame);
                    }
                    remaining_gap_amount -= count * tier_value;
                }

                // 4.b.3.3 If we've covered the gap amount, we're done.
                if remaining_gap_amount == 0 {
                    break;
                }
            }

            // 4.b.4 If there's still remaining gap amount, we need to round up by adding one more of the smallest available tier.
            if remaining_gap_amount > 0 {
                // 4.b.4.1 Use the smallest available tier we stored earlier.
                if let Some((_, script_pubkey, flame_tier)) = smallest_tier {
                    let flame = Flame::new(flame_tier, script_pubkey);
                    flames.push(flame);
                }
            }
        }
    }

    // 5 Return the list of flames to return.
    Some(flames)
}
