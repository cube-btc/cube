use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use colored::Colorize;

/// Prints an account's coin balance in satoshis from the coin manager (`coins` with no args uses the node's self account).
pub async fn coins_command(coin_manager: &COIN_MANAGER, account_key: [u8; 32]) {
    let balance = {
        let cm = coin_manager.lock().await;
        cm.get_account_balance(account_key)
    };
    match balance {
        Some(b) => println!("{}", b),
        None => eprintln!(
            "{}",
            "No coin balance for this account in the coin manager (account not registered)."
                .yellow()
        ),
    }
}
