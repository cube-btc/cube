use crate::{
    executive::stack::{
        stack_error::{CoinBalanceGetError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{StackItemUintExt, StackUint},
    },
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Pushes the account's individual BTC balance into the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_EXT_BALANCE;

impl OP_EXT_BALANCE {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the kind.
        let kind_item = stack_holder.pop()?;

        // Match the kind.
        match kind_item.is_false() {
            // Interpret as account key.
            true => {
                // Pop the account key.
                let account_key = stack_holder.pop()?;

                // Convert the account key to bytes.
                let account_key_bytes: [u8; 32] = match account_key.bytes().try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StackError::CoinBalanceGetError(
                            CoinBalanceGetError::InvalidAccountKeyBytes(
                                account_key.bytes().to_vec(),
                            ),
                        ))
                    }
                };

                // Get the account coin holder.
                let account_coin_holder = {
                    let _coin_holder = coin_holder.lock().await;
                    _coin_holder.account_coin_holder()
                };

                // Get the account balance.
                let account_balance = {
                    let _account_coin_holder = account_coin_holder.lock().await;
                    _account_coin_holder
                        .get_account_balance(account_key_bytes)
                        .ok_or(StackError::CoinBalanceGetError(
                            CoinBalanceGetError::UnableToGetAccountBalance(account_key_bytes),
                        ))?
                };

                // Convert the account balance to a stack uint.
                let account_balance_as_stack_uint = StackUint::from(account_balance);

                // Convert the account balance to a stack item.
                let account_balance_as_stack_item =
                    StackItem::from_stack_uint(account_balance_as_stack_uint);

                // Push the account balance to the stack.
                stack_holder.push(account_balance_as_stack_item)?;
            }
            // Interpret as contract id.
            false => {
                // Pop the contract id.
                let contract_id = stack_holder.pop()?;

                // Convert the contract id to bytes.
                let contract_id_bytes: [u8; 32] = match contract_id.bytes().try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StackError::CoinBalanceGetError(
                            CoinBalanceGetError::InvalidContractIdBytes(
                                contract_id.bytes().to_vec(),
                            ),
                        ))
                    }
                };

                // Get the contract coin holder.
                let contract_coin_holder = {
                    let _coin_holder = coin_holder.lock().await;
                    _coin_holder.contract_coin_holder()
                };

                // Get the contract balance.
                let contract_balance = {
                    let _contract_coin_holder = contract_coin_holder.lock().await;
                    _contract_coin_holder
                        .get_contract_balance(contract_id_bytes)
                        .ok_or(StackError::CoinBalanceGetError(
                            CoinBalanceGetError::UnableToGetContractBalance(contract_id_bytes),
                        ))?
                };
                // Convert the contract balance to a stack uint.
                let contract_balance_as_stack_uint = StackUint::from(contract_balance);

                // Convert the contract balance to a stack item.
                let contract_balance_as_stack_item =
                    StackItem::from_stack_uint(contract_balance_as_stack_uint);

                // Push the contract balance to the stack.
                stack_holder.push(contract_balance_as_stack_item)?;
            }
        }

        Ok(())
    }

    /// Returns the bytecode for the `OP_EXT_BALANCE` opcode (0xc0).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc0]
    }
}
