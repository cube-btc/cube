#[cfg(test)]
mod coin_manager_tests {
    use cube::inscriptive::coin_manager::coin_manager::{
        erase_coin_manager, CoinManager, COIN_MANAGER,
    };
    use cube::operative::Chain;

    // First account key.
    const ACCOUNT_KEY_1: [u8; 32] = [
        0xfc, 0x7a, 0xca, 0xce, 0xf4, 0x50, 0x95, 0x60, 0x04, 0x27, 0xc6, 0x16, 0x87, 0x4a, 0x96,
        0xb7, 0x0e, 0x16, 0xcd, 0x2a, 0xb2, 0xa0, 0xea, 0x31, 0xa4, 0xa6, 0xae, 0x83, 0x4d, 0xbf,
        0x6f, 0x9d,
    ];

    // Second account key.
    const ACCOUNT_KEY_2: [u8; 32] = [
        0x30, 0x7b, 0xe2, 0x81, 0x2b, 0x63, 0x4e, 0xca, 0x77, 0x2c, 0xc4, 0xcc, 0x3c, 0xe1, 0x4b,
        0x57, 0xa7, 0xd7, 0x6e, 0x66, 0x80, 0x09, 0xc5, 0xc6, 0x9e, 0xe6, 0xa6, 0x67, 0x7a, 0x95,
        0x33, 0x8b,
    ];

    // Third account key.
    const ACCOUNT_KEY_3: [u8; 32] = [
        0xa9, 0xd0, 0xe7, 0xe0, 0xa7, 0x78, 0xdc, 0x11, 0x84, 0xf8, 0xa5, 0x00, 0x3c, 0x50, 0xf2,
        0xd0, 0x9f, 0xfe, 0xa0, 0x93, 0x60, 0xe6, 0x71, 0x66, 0x8f, 0x8e, 0xb4, 0xd5, 0x18, 0x56,
        0x51, 0x17,
    ];

    // First contract ID.
    const CONTRACT_ID_1: [u8; 32] = [
        0xe4, 0xff, 0x5e, 0x7d, 0x7a, 0x7f, 0x08, 0xe9, 0x80, 0x0a, 0x3e, 0x25, 0xcb, 0x77, 0x45,
        0x33, 0xcb, 0x20, 0x04, 0x0d, 0xf3, 0x0b, 0x6b, 0xa1, 0x0f, 0x95, 0x6f, 0x9a, 0xcd, 0x0e,
        0xb3, 0xf7,
    ];

    // Second contract ID.
    const CONTRACT_ID_2: [u8; 32] = [
        0xd1, 0xbb, 0xd7, 0x3b, 0xb0, 0x91, 0x90, 0xbf, 0xb8, 0x83, 0x05, 0x67, 0x71, 0xe2, 0x2e,
        0x99, 0x75, 0x41, 0xed, 0x20, 0x07, 0x97, 0x93, 0xbf, 0x33, 0x97, 0x5f, 0xe1, 0x65, 0x45,
        0x81, 0xc3,
    ];

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

        // 5 First account operations.
        {
            // 5.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 5.2 Register the account.
            let result = _coin_manager.register_account(ACCOUNT_KEY_1, 0);

            // 5.3 The result should be ok.
            assert!(result.is_ok());

            // 5.4 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(ACCOUNT_KEY_1);

            // 5.5 The account should not be registered yet because changes are not applied yet.
            assert_eq!(is_registered, false);

            // 5.6 Apply the changes.
            let result = _coin_manager.apply_changes();

            // 5.7 The result should be ok.
            assert!(result.is_ok());

            // 5.9 Flush the delta.
            _coin_manager.flush_delta();

            // 5.11 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(ACCOUNT_KEY_1);
            assert_eq!(is_registered, true);

            // 5.12 Check if the account balance is 0.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(0));

            // 5.13 Try to register the account again.
            let result = _coin_manager.register_account(ACCOUNT_KEY_1, 0);

            // 5.14 The result should be an error.
            assert!(result.is_err());

            // 5.15 Account balance up.
            let result = _coin_manager.account_balance_up(ACCOUNT_KEY_1, 1000);

            // 5.16 The result should be ok.
            assert!(result.is_ok());

            // 5.17 Check if the account balance is 1000.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(1000));

            // 5.18 Flush the delta without applying the changes.
            _coin_manager.flush_delta();

            // 5.19 The account balance should be back to 0.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(0));

            // 5.20 Account balance up again.
            let result = _coin_manager.account_balance_up(ACCOUNT_KEY_1, 5000);

            // 5.21 The result should be ok.
            assert!(result.is_ok());

            // 5.22 Check if the account balance is 5000.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(5000));

            // 5.23 This time apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 5.24 Flush the delta.
            _coin_manager.flush_delta();

            // 5.25 Check if the account balance is still 5000.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(5000));

            // 5.26 Account balance down.
            let result = _coin_manager.account_balance_down(ACCOUNT_KEY_1, 2000);

            // 5.27 The result should be ok.
            assert!(result.is_ok());

            // 5.28 Check if the account balance is 3000.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(3000));

            // 5.29 Balance up again.
            let result = _coin_manager.account_balance_up(ACCOUNT_KEY_1, 250);
            assert!(result.is_ok());

            // 5.30 Check if the account balance is 3250.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_1);
            assert_eq!(account_balance, Some(3250));

            // 5.31 Account balance down excessively. This should fail.
            let result = _coin_manager.account_balance_down(ACCOUNT_KEY_1, 3251);
            assert!(result.is_err());

            // 5.32 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 5.33 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 6 Second account operations.
        {
            // 6.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 6.2 Register the account with initial balance 500.
            let result = _coin_manager.register_account(ACCOUNT_KEY_2, 500);
            assert!(result.is_ok());

            // 6.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 6.4 Flush the delta.
            _coin_manager.flush_delta();

            // 6.5 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(ACCOUNT_KEY_2);
            assert_eq!(is_registered, true);

            // 6.6 Check if the account balance is 500.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_2);
            assert_eq!(account_balance, Some(500));

            // 6.7 Account balance down.
            let result = _coin_manager.account_balance_down(ACCOUNT_KEY_2, 100);
            assert!(result.is_ok());

            // 6.8 Check if the account balance is 400.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_2);
            assert_eq!(account_balance, Some(400));

            // 6.9 Balance down again.
            let result = _coin_manager.account_balance_down(ACCOUNT_KEY_2, 50);
            assert!(result.is_ok());

            // 6.10 Check if the account balance is 350.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_2);
            assert_eq!(account_balance, Some(350));

            // 6.11 Balance up.
            let result = _coin_manager.account_balance_up(ACCOUNT_KEY_2, 1000);
            assert!(result.is_ok());

            // 6.12 Check if the account balance is 1350.
            let account_balance = _coin_manager.get_account_balance(ACCOUNT_KEY_2);
            assert_eq!(account_balance, Some(1350));

            // 6.13 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 6.14 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 7 Third account operations.
        {
            // 7.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 7.2 Register the account.
            let result = _coin_manager.register_account(ACCOUNT_KEY_3, 0);
            assert!(result.is_ok());

            // 7.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 7.4 Flush the delta.
            _coin_manager.flush_delta();

            // 7.5 Check if the account is registered.
            let is_registered = _coin_manager.is_account_registered(ACCOUNT_KEY_3);
            assert_eq!(is_registered, true);
        }

        // 8 Register a contract.
        {
            // 8.2 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 8.3 Register the contract with initial balance 100,000.
            let result = _coin_manager.register_contract(CONTRACT_ID_1, 100_000);
            assert!(result.is_ok());

            // 8.7 Check if the contract is registered. Should fail since changes are not applied yet.
            let is_registered = _coin_manager.is_contract_registered(CONTRACT_ID_1);
            assert_eq!(is_registered, false);

            // 8.8 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 8.9 Flush the delta.
            _coin_manager.flush_delta();

            // 8.10 Check if the contract is registered.
            let is_registered = _coin_manager.is_contract_registered(CONTRACT_ID_1);
            assert_eq!(is_registered, true);
        }

        // 9 Contract balance updates.
        {
            // 9.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 9.2 Contract balance up by 5000.
            let result = _coin_manager.contract_balance_up(CONTRACT_ID_1, 5000);
            assert!(result.is_ok());

            // 9.3 Check if the contract balance is 105,000.
            let contract_balance = _coin_manager.get_contract_balance(CONTRACT_ID_1);
            assert_eq!(contract_balance, Some(105_000));

            // 9.4 Contract balance down by 2000.
            let result = _coin_manager.contract_balance_down(CONTRACT_ID_1, 2000);
            assert!(result.is_ok());

            // 9.5 Check if the contract balance is 103,000.
            let contract_balance = _coin_manager.get_contract_balance(CONTRACT_ID_1);
            assert_eq!(contract_balance, Some(103_000));

            // 9.6 Excessive contract balance down. This should fail.
            let result = _coin_manager.contract_balance_down(CONTRACT_ID_1, 103_001);
            assert!(result.is_err());

            // 9.7 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 9.8 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 10 Allocate the first account in the contract shadow space.
        {
            // 10.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 10.2 Allocate the first account in the contract shadow space.
            let result = _coin_manager.contract_shadow_alloc_account(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert!(result.is_ok());

            // 10.3 Get alloc value in sati-satoshis. Initially it should be zero.
            let alloc_value =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(alloc_value, Some(0));

            // 10.4 Try to allocate the account again. This should fail.
            let result = _coin_manager.contract_shadow_alloc_account(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert!(result.is_err());

            // 10.5 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 10.6 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 11 Try to get alloc value from a non-allocated account.
        {
            // 11.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 11.2 Get alloc value in sati-satoshis. Should be none.
            let alloc_value =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(alloc_value, None);
        }

        // 12 Try to read balance of a non-registered contract.
        {
            // 12.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 12.2 Get contract balance. Should be none.
            let contract_balance = _coin_manager.get_contract_balance(CONTRACT_ID_2);
            assert_eq!(contract_balance, None);
        }

        // 13 Shadow up.
        {
            // 13.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 13.2 Shadow up by 1000.
            let result = _coin_manager.shadow_up(CONTRACT_ID_1, ACCOUNT_KEY_1, 1000);
            assert!(result.is_ok());

            // 13.3 Check if shadow alloc value is 1000.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(1000));

            // 13.4 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 13.5 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 14 Shadow down.
        {
            // 14.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 14.2 Shadow down by 500.
            let result = _coin_manager.shadow_down(CONTRACT_ID_1, ACCOUNT_KEY_1, 500);
            assert!(result.is_ok());

            // 13.5 Check if shadow alloc value is 500.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(500));

            // 13.6 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 13.7 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 15 Shadow up all.
        {
            // 15.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 15.2 Shadow up all by 25.
            let result = _coin_manager.shadow_up_all(CONTRACT_ID_1, 25);
            assert!(result.is_ok());

            // 15.3 Check if shadow alloc value is 525.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(525));

            // 15.4 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 15.5 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 16 Shadow down all.
        {
            // 16.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 16.2 Shadow down all by 100.
            let result = _coin_manager.shadow_down_all(CONTRACT_ID_1, 100);
            assert!(result.is_ok());

            // 16.3 Check if shadow alloc value is 425.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(425));

            // 16.4 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 16.5 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 17 Allocate the second account in the contract shadow space.
        {
            // 17.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 17.2 Allocate the second account in the contract shadow space.
            let result = _coin_manager.contract_shadow_alloc_account(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert!(result.is_ok());

            // 17.3 Get alloc value in sati-satoshis. Initially it should be zero.
            let alloc_value =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(alloc_value, Some(0));

            // 17.4 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 17.5 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 18 Shadow up all.
        {
            // 18.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 18.2 Shadow up all by 25.
            let result = _coin_manager.shadow_up_all(CONTRACT_ID_1, 100);
            assert!(result.is_ok());

            // 18.3 First account shadow alloc value should be 525.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(525));

            // 18.4 Second account shadow alloc value should remain zero.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value, Some(0));

            // 18.5 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 18.6 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 19 Shadow down all.
        {
            // 19.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 19.2 Shadow down all by 50.
            let result = _coin_manager.shadow_down_all(CONTRACT_ID_1, 1);
            assert!(result.is_ok());

            // 19.3 First account shadow alloc value should be 524.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value, Some(524));

            // 19.4 Second account shadow alloc value should remain zero.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value, Some(0));

            // 19.5 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 19.6 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 20 Shadow up second account.
        {
            // 20.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 20.2 Shadow up second account by 5.
            let result = _coin_manager.shadow_up(CONTRACT_ID_1, ACCOUNT_KEY_2, 5);
            assert!(result.is_ok());

            // 20.3 Check if shadow alloc value is 5.
            let shadow_alloc_value =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value, Some(5));

            // 20.4 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 20.5 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 21 Shadow up all.
        {
            // 21.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 21.2 Shadow up all by 100.
            let result = _coin_manager.shadow_up_all(CONTRACT_ID_1, 100);
            assert!(result.is_ok());

            // 21.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 21.4 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 22 Proportioning checks.
        {
            // 22.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 22.2 Get shadow alloc value of first account in sati-satoshis.
            let shadow_alloc_value_in_sati_satoshis =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value_in_sati_satoshis, Some(62305482041));

            // 22.3 Get shadow alloc value of first account in satoshis.
            let shadow_alloc_value_in_satoshis =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value_in_satoshis, Some(623)); // Proportionally increased by a little over than 99 satoshis.

            // 22.4 Get shadow alloc value of second account in sati-satoshis.
            let shadow_alloc_value_in_sati_satoshis =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value_in_sati_satoshis, Some(594517958));

            // 22.5 Get shadow alloc value of second account in satoshis.
            let shadow_alloc_value_in_satoshis =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value_in_satoshis, Some(5)); // Proportionally increased by slighly less than 1 satoshi.
        }

        // 23 Shadow up all again.
        {
            // 23.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 23.2 Shadow up all by 1000.
            let result = _coin_manager.shadow_up_all(CONTRACT_ID_1, 1000);
            assert!(result.is_ok());

            // 23.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 23.4 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 24 Proportioning checks.
        {
            // 24.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 24.2 Get shadow alloc value of first account in sati-satoshis.
            let shadow_alloc_value_in_sati_satoshis =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value_in_sati_satoshis, Some(161360302455));

            // 24.3 Get shadow alloc value of first account in satoshis.
            let shadow_alloc_value_in_satoshis =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_value_in_satoshis, Some(1613)); // Proportionally increased by 990.

            // 24.4 Get shadow alloc value of second account in sati-satoshis.
            let shadow_alloc_value_in_sati_satoshis =
                _coin_manager.get_shadow_alloc_value_in_sati_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value_in_sati_satoshis, Some(1539697541));

            // 24.5 Get shadow alloc value of second account in satoshis.
            let shadow_alloc_value_in_satoshis =
                _coin_manager.get_shadow_alloc_value_in_satoshis(CONTRACT_ID_1, ACCOUNT_KEY_2);
            assert_eq!(shadow_alloc_value_in_satoshis, Some(15)); // Proportionally increased by 10.
        }

        // 25 Register the second contract.
        {
            // 25.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 25.2 Register the seconds contract.
            let result = _coin_manager.register_contract(CONTRACT_ID_2, 10_000);
            assert!(result.is_ok());

            // 25.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 25.4 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 26 Allocate the first account in the second contract shadow space.
        {
            // 26.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 26.2 Allocate the first account in the second contract shadow space.
            let result = _coin_manager.contract_shadow_alloc_account(CONTRACT_ID_2, ACCOUNT_KEY_1);
            assert!(result.is_ok());

            // 26.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 26.4 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 27 Check shadow alloc overall sum of the first contract.
        {
            // 27.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 27.2 Get shadow alloc overall sum of the first contract.
            let shadow_alloc_overall_sum =
                _coin_manager.get_account_global_shadow_allocs_sum_in_satoshis(ACCOUNT_KEY_1);

            assert_eq!(shadow_alloc_overall_sum, Some(1613));
        }

        // 28 Shadow up first account in second contract.
        {
            // 28.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 28.2 Shadow up first account in second contract by 3.
            let result = _coin_manager.shadow_up(CONTRACT_ID_2, ACCOUNT_KEY_1, 3);
            assert!(result.is_ok());

            // 28.3 Apply changes.
            let result = _coin_manager.apply_changes();
            assert!(result.is_ok());

            // 28.4 Flush the delta.
            _coin_manager.flush_delta();
        }

        // 29 Check (again) shadow alloc overall sum of the first contract.
        {
            // 29.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 29.2 Get shadow alloc overall sum of the first contract.
            let shadow_alloc_overall_sum =
                _coin_manager.get_account_global_shadow_allocs_sum_in_satoshis(ACCOUNT_KEY_1);
            assert_eq!(shadow_alloc_overall_sum, Some(1616)); // Has increased by 3.
        }

        //println!("Coin manager y: {}", coin_manager.lock().await.json());

        Ok(())
    }
}
