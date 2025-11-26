#[cfg(test)]
mod state_manager_tests {
    use cube::inscriptive::state_manager::state_manager::{
        erase_state_manager, StateManager, STATE_MANAGER,
    };
    use cube::operative::Chain;

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

    // First state key and value.
    const STATE_KEY_1: [u8; 32] = [0xaau8; 32];
    const STATE_VALUE_1: [u8; 32] = [0xbbu8; 32];

    // Second state key and value.
    const STATE_KEY_2: [u8; 32] = [0xccu8; 32];
    const STATE_VALUE_2: [u8; 32] = [0xddu8; 32];

    // Third state key and value.
    const STATE_KEY_3: [u8; 32] = [0xeeu8; 32];
    const STATE_VALUE_3: [u8; 32] = [0xffu8; 32];

    #[tokio::test]
    async fn state_manager_tests() -> Result<(), String> {
        // 1 Set the chain for local tests.
        let chain = Chain::Testbed;

        // 2 Erase first the state manager.
        erase_state_manager(chain);

        // 3 Construct the state manager.
        let state_manager: STATE_MANAGER = StateManager::new(chain).unwrap();

        // 4 Pre-execution.
        {
            state_manager.lock().await.pre_execution();
        }

        // 5 Register the first contract.
        {
            // 5.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 5.2 Register the first contract.
            let result = _state_manager.register_contract(CONTRACT_ID_1);
            assert!(result.is_ok());
        }

        // 6 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 7 Insert the first state.
        {
            // 7.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 7.2 Insert the state.
            let result = _state_manager.insert_update_state(
                CONTRACT_ID_1,
                &Vec::from(STATE_KEY_1),
                &Vec::from(STATE_VALUE_1),
                false,
            );
            assert!(result.is_ok());
        }

        // 8 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 9 Insert the second state.
        {
            // 9.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 9.2 Insert the state.
            let result = _state_manager.insert_update_state(
                CONTRACT_ID_1,
                &Vec::from(STATE_KEY_2),
                &Vec::from(STATE_VALUE_2),
                false,
            );
            assert!(result.is_ok());
        }

        // 10 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 11 Read the first state.
        {
            // 11.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 11.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_1));
            assert!(result.is_some());

            // 11.3 Assert match.
            assert_eq!(result.unwrap(), STATE_VALUE_1);
        }

        // 12 Pre-execution before rollback test.
        {
            state_manager.lock().await.pre_execution();
        }

        // 13 Insert the third state.
        {
            // 13.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 13.2 Insert the state.
            let result = _state_manager.insert_update_state(
                CONTRACT_ID_1,
                &Vec::from(STATE_KEY_3),
                &Vec::from(STATE_VALUE_3),
                false,
            );
            assert!(result.is_ok());
        }

        // NOTE: This time we are not applying changes.

        // 14 Try to read the third state.
        {
            // 14.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 14.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_3));
            assert!(result.is_some());

            // 14.3 Assert match.
            assert_eq!(result.unwrap(), STATE_VALUE_3);
        }

        // 15 Rollback the changes.
        {
            // 15.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 15.2 Restore old states by rolling back.
            _state_manager.rollback_last();
        }

        // 16 Try to read the third state again post-rollback.
        {
            // 16.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 16.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_3));

            // 16.3 Should be none as the state was rolled back.
            assert!(result.is_none());
        }

        // 17 Insert the third state again.
        {
            // 17.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 17.2 Insert the state.
            let result = _state_manager.insert_update_state(
                CONTRACT_ID_1,
                &Vec::from(STATE_KEY_3),
                &Vec::from(STATE_VALUE_3),
                false,
            );
            assert!(result.is_ok());
        }

        // 18 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 19 Remove the second state.
        {
            // 19.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 19.2 Remove the state.
            let result = _state_manager.remove_state(CONTRACT_ID_1, &Vec::from(STATE_KEY_2), false);
            assert!(result.is_ok());
        }

        // 20 Try to read the second state post-removal.
        {
            // 20.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 20.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_2));

            // 20.3 Should be none as the state was removed.
            assert!(result.is_none());
        }

        // 21 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 22 Pre-execution before second rollback test.
        {
            state_manager.lock().await.pre_execution();
        }

        // 23 Remove the first state.
        {
            // 23.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 23.2 Remove the state.
            let result = _state_manager.remove_state(CONTRACT_ID_1, &Vec::from(STATE_KEY_1), false);
            assert!(result.is_ok());
        }

        // NOTE: This time we are not applying changes.

        // 24 Try to read the first state post-removal.
        {
            // 24.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 24.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_1));

            // 24.3 Should be none as the state was removed.
            assert!(result.is_none());
        }

        // 25 Rollback the changes.
        {
            // 25.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 25.2 Restore old states by rolling back.
            _state_manager.rollback_last();
        }

        // 26 Try to read the first state again post-rollback.
        {
            // 26.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 26.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_1, &Vec::from(STATE_KEY_1));

            // 26.3 Should be some as the removal was rolled back.
            assert!(result.is_some());

            // 26.4 Assert match.
            assert_eq!(result.unwrap(), STATE_VALUE_1);
        }

        // 27 Register the second contract.
        {
            // 27.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 27.2 Register the second contract.
            let result = _state_manager.register_contract(CONTRACT_ID_2);
            assert!(result.is_ok());
        }

        // 28 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 29 Insert the third state into the second contract.
        {
            // 29.1 Lock the state manager.
            let mut _state_manager = state_manager.lock().await;

            // 29.2 Insert the state.
            let result = _state_manager.insert_update_state(
                CONTRACT_ID_2,
                &Vec::from(STATE_KEY_3),
                &Vec::from(STATE_VALUE_3),
                false,
            );
            assert!(result.is_ok());
        }

        // 30 Apply changes.
        let result = state_manager.lock().await.apply_changes();
        assert!(result.is_ok());

        // 31 Try to read the third state from the second contract.
        {
            // 31.1 Lock the state manager.
            let _state_manager = state_manager.lock().await;

            // 31.2 Read the state.
            let result = _state_manager.get_state_value(CONTRACT_ID_2, &Vec::from(STATE_KEY_3));
            assert!(result.is_some());

            // 31.3 Assert match.
            assert_eq!(result.unwrap(), STATE_VALUE_3);
        }

        //println!("{}", state_manager.lock().await.json());

        Ok(())
    }
}
