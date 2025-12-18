use crate::inscriptive::flame_manager::flame::flame::Flame;
use crate::inscriptive::flame_manager::flame_manager::FlameIndex;
use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Sorts the flames.
pub fn sort_flames(
    flames_to_sort: HashMap<AccountKey, Vec<Flame>>,
) -> Vec<(AccountKey, Vec<(FlameIndex, Flame)>)> {
    // 1 Check if the flames to sort is empty.
    if flames_to_sort.is_empty() {
        return Vec::new();
    }

    // 2 Collect all account keys and sort them lexicographically.
    let mut account_keys: Vec<AccountKey> = flames_to_sort.keys().cloned().collect();
    account_keys.sort();

    // 3 Sort flames within each account by flame value (descending), then lexicographically if equal.
    let mut sorted_flames_by_account: HashMap<AccountKey, Vec<Flame>> = HashMap::new();
    for (account_key, flames) in flames_to_sort.iter() {
        let mut sorted_flames = flames.clone();
        sorted_flames.sort_by(|flame_a, flame_b| {
            // First, compare by flame value (descending - higher values first).
            match flame_b.satoshi_amount().cmp(&flame_a.satoshi_amount()) {
                std::cmp::Ordering::Equal => {
                    // If flame values are equal, compare lexicographically by their byte representation (ascending).
                    flame_a.to_bytes().cmp(&flame_b.to_bytes())
                }
                other => other,
            }
        });
        sorted_flames_by_account.insert(account_key.to_owned(), sorted_flames);
    }

    // 4 Assign sequential flame indexes across all accounts.
    // Flames from account A get indexes 0, 1, 2..., then account B continues with +1, etc.
    let mut result: Vec<(AccountKey, Vec<(FlameIndex, Flame)>)> = Vec::new();
    let mut current_index: FlameIndex = 0;

    for account_key in account_keys.iter() {
        let flames = sorted_flames_by_account.get(account_key).unwrap();
        let mut account_flames: Vec<(FlameIndex, Flame)> = Vec::new();

        for flame in flames.iter() {
            account_flames.push((current_index, flame.clone()));
            current_index += 1;
        }

        result.push((account_key.to_owned(), account_flames));
    }

    // 5 Return the result.
    result
}
