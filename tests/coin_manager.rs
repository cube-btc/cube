#[cfg(test)]
mod coin_manager_tests {
    use cube::inscriptive::coin_manager::coin_manager::{
        erase_coin_manager, CoinManager, COIN_MANAGER,
    };
    use cube::operative::Chain;

    #[tokio::test]
    async fn coin_manager_tests() -> Result<(), String> {
        // 1 Set the chain for local tests.
        let chain = Chain::Testbed;

        // 2 Erase first the coin manager.
        erase_coin_manager(chain);

        // 3 Construct the coin manager.
        let coin_manager: COIN_MANAGER = CoinManager::new(chain).unwrap();

        // 4 Registering an account with special keys should fail.
        {
            // 4.1 Special db key 1.
            let special_db_key_1 = [0x00; 32];
            // 4.2 Special db key 2.
            let special_db_key_2 = [0x01; 32];

            // 4.3 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 4.4 Register the account.
            let result_1 = _coin_manager.register_account(special_db_key_1, 0);
            let result_2 = _coin_manager.register_account(special_db_key_2, 0);

            // 4.5 The result should be an error.
            assert!(result_1.is_err());
            assert!(result_2.is_err());
        }

        // 5 Register an account.
        {
            // 5.1 Random account key.
            let account_key: [u8; 32] =
                hex::decode("fc7acacef45095600427c616874a96b70e16cd2ab2a0ea31a4a6ae834dbf6f9d")
                    .expect("This should never fail.")
                    .try_into()
                    .expect("This should never fail.");

            // 5.2 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 5.3 Register the account.
            let result = _coin_manager.register_account(account_key, 0);

            // 5.4 The result should be ok.
            assert!(result.is_ok());

            // 5.5 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(account_key);

            // 5.6 The account should not be registered yet because changes are not applied yet.
            assert_eq!(is_registered, false);

            // 5.7 Apply the changes.
            let result = _coin_manager.apply_changes();

            // 5.8 Flush the delta.
            _coin_manager.flush_delta();

            // 5.9 The result should be ok.
            assert!(result.is_ok());

            // 5.10 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(account_key);
            assert_eq!(is_registered, true);

            // 5.11 Check if the account balance is 0.
            let account_balance = _coin_manager.get_account_balance(account_key);
            assert_eq!(account_balance, Some(0));

            // 5.12 Try to register the account again.
            let result = _coin_manager.register_account(account_key, 0);

            // 5.13 The result should be an error.
            assert!(result.is_err());
        }

        // Print the coin manager state.
        println!("Coin manager: {}", coin_manager.lock().await.json());

        Ok(())
    }
}
