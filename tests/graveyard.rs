#[cfg(test)]
mod graveyard_tests {
    use cube::inscriptive::graveyard::errors::redeem_account_coins_error::GraveyardRedeemAccountCoinsError;
    use cube::inscriptive::graveyard::graveyard::{erase_graveyard, Graveyard, GRAVEYARD};
    use cube::operative::Chain;

    // 1.a First account to be burried.
    const ACCOUNT_KEY_1: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    // 1.b Redemption amount for the first account.
    const REDEMPTION_AMOUNT_1: u64 = 1_000;

    // 2.a Second account to be burried.
    const ACCOUNT_KEY_2: [u8; 32] = [
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    // 2.b Redemption amount for the second account.
    const REDEMPTION_AMOUNT_2: u64 = 5_000;

    // 3.a Third account to be burried.
    const ACCOUNT_KEY_3: [u8; 32] = [
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    // 3.b Redemption amount for the third account.
    const REDEMPTION_AMOUNT_3: u64 = 10_000;

    #[tokio::test]
    async fn graveyard_tests() -> Result<(), String> {
        // 1 Set the chain for local tests.
        let chain = Chain::Testbed;

        // 2 Erase first the graveyard.
        erase_graveyard(chain);

        // 3 Construct the graveyard.
        let graveyard: GRAVEYARD = Graveyard::new(chain)
            .map_err(|err| format!("Error constructing graveyard: {:?}", err))?;

        // 4 Pre-execution.
        {
            // 4.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 4.2 Pre-execution.
            _graveyard.pre_execution();
        }

        // 5 Burry the first account.
        {
            // 5.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 5.2 Burry the first account.
            _graveyard
                .burry_account(ACCOUNT_KEY_1, REDEMPTION_AMOUNT_1)
                .map_err(|err| format!("Error burrying account: {:?}", err))?;
        }

        // 6 Check if the account is burried.
        {
            // 6.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 6.2 Check if the account is burried.
            assert_eq!(_graveyard.is_account_burried(ACCOUNT_KEY_1), true);
        }

        // 7 Retrieve the redemption amount for the account.
        {
            // 7.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 7.2 Retrieve the redemption amount for the account.
            let redemption_amount =
                _graveyard
                    .get_redemption_amount(ACCOUNT_KEY_1)
                    .ok_or(format!(
                        "Error retrieving redemption amount for account: {:?}",
                        ACCOUNT_KEY_1
                    ))?;

            // 7.3 Check if the redemption amount is correct.
            assert_eq!(redemption_amount, REDEMPTION_AMOUNT_1);
        }

        // 8 Burry the second account.
        {
            // 8.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 8.2 Burry the second account.
            _graveyard
                .burry_account(ACCOUNT_KEY_2, REDEMPTION_AMOUNT_2)
                .map_err(|err| format!("Error burrying account: {:?}", err))?;
        }

        // 9 Check if the account is burried.
        {
            // 9.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 9.2 Check if the account is burried.
            assert_eq!(_graveyard.is_account_burried(ACCOUNT_KEY_2), true);
        }

        // 10 Retrieve the redemption amount for the account.
        {
            // 10.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 10.2 Retrieve the redemption amount for the account.
            let redemption_amount =
                _graveyard
                    .get_redemption_amount(ACCOUNT_KEY_2)
                    .ok_or(format!(
                        "Error retrieving redemption amount for account: {:?}",
                        ACCOUNT_KEY_2
                    ))?;

            // 10.3 Check if the redemption amount is correct.
            assert_eq!(redemption_amount, REDEMPTION_AMOUNT_2);
        }

        // 11 Try to redeem the first account coins: expect an error because the burrying was epheremal and changes were not applied yet.
        {
            // 11.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 11.2 Redeem the first account coins.
            let result: Result<u64, GraveyardRedeemAccountCoinsError> =
                _graveyard.redeem_account_coins(ACCOUNT_KEY_1);

            // 11.3 Check if the result is an error.
            assert!(result.is_err());

            // 11.4 Check if the error is correct.
            assert!(matches!(
                result.err().unwrap(),
                GraveyardRedeemAccountCoinsError::ThisAccountHasJustBeenEphemerallyBurried(_)
            ));
        }

        // 12 Now apply the changes to be able to redeem the account coins.
        {
            // 12.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 12.2 Apply the changes.
            _graveyard
                .apply_changes()
                .map_err(|err| format!("Error applying changes: {:?}", err))?;
        }

        // 13 Now try to redeem the first account coins again: this should succeed.
        {
            // 13.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 13.2 Redeem the first account coins.
            let redemption_amount = _graveyard
                .redeem_account_coins(ACCOUNT_KEY_1)
                .map_err(|err| format!("Error redeeming account coins: {:?}", err))?;

            // 13.3 Check if the redemption amount is correct.
            assert_eq!(redemption_amount, REDEMPTION_AMOUNT_1);
        }

        // 14 Not try to redeem again, just to expect error for double redemption.
        {
            // 14.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 14.2 Redeem the first account coins.
            let result: Result<u64, GraveyardRedeemAccountCoinsError> =
                _graveyard.redeem_account_coins(ACCOUNT_KEY_1);

            // 14.3 Check if the result is an error.
            assert!(result.is_err());

            // 14.4 Check if the error is correct.
            assert!(matches!(
                result.err().unwrap(),
                GraveyardRedeemAccountCoinsError::AccountCoinsHasJustBeenEphemerallyRedeemed(_)
            ));
        }

        // 15 Now get the first account redemption amount again: it should be zero because the coins have been redeemed.
        {
            // 15.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 15.2 Get the first account redemption amount.
            let redemption_amount =
                _graveyard
                    .get_redemption_amount(ACCOUNT_KEY_1)
                    .ok_or(format!(
                        "Error retrieving redemption amount for account: {:?}",
                        ACCOUNT_KEY_1
                    ))?;

            // 15.3 Check if the redemption amount is correct.
            assert_eq!(redemption_amount, 0);
        }

        // 16 Apply changes.
        {
            // 16.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 16.2 Apply the changes.
            _graveyard
                .apply_changes()
                .map_err(|err| format!("Error applying changes: {:?}", err))?;
        }

        // 17 Burry the third account.
        {
            // 17.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 17.2 Burry the third account.
            _graveyard
                .burry_account(ACCOUNT_KEY_3, REDEMPTION_AMOUNT_3)
                .map_err(|err| format!("Error burrying account: {:?}", err))?;
        }

        // 18 Check if the account is burried.
        {
            // 18.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 18.2 Check if the account is burried.
            assert_eq!(_graveyard.is_account_burried(ACCOUNT_KEY_3), true);
        }

        // 19 Retrieve the redemption amount for the account.
        {
            // 19.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 19.2 Retrieve the redemption amount for the account.
            let redemption_amount =
                _graveyard
                    .get_redemption_amount(ACCOUNT_KEY_3)
                    .ok_or(format!(
                        "Error retrieving redemption amount for account: {:?}",
                        ACCOUNT_KEY_3
                    ))?;

            // 19.3 Check if the redemption amount is correct.
            assert_eq!(redemption_amount, REDEMPTION_AMOUNT_3);
        }

        // 20 Apply changes.
        {
            // 20.1 Lock the graveyard.
            let mut _graveyard = graveyard.lock().await;

            // 20.2 Apply the changes.
            _graveyard
                .apply_changes()
                .map_err(|err| format!("Error applying changes: {:?}", err))?;
        }

        // Print the graveyard.
        //println!("Graveyard: {:?}", graveyard.lock().await.json());

        // 21 Return the result.
        Ok(())
    }
}
