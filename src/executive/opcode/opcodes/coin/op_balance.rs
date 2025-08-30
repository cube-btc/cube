use crate::{
    executive::stack::{
        stack_error::{CoinOpsError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Pushes the BTC balance of an account or a contract into the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_BALANCE;

impl OP_BALANCE {
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

        // Convert the kind to a stack uint.
        let kind_as_stack_uint = match kind_item.to_stack_uint() {
            Some(stack_uint) => stack_uint,
            None => {
                return Err(StackError::CoinOpsError(CoinOpsError::InvalidKindBytes(
                    kind_item.bytes().to_vec(),
                )))
            }
        };

        // Convert the kind to a usize.
        let kind_as_usize = match kind_as_stack_uint.to_usize() {
            Some(usize) => usize,
            None => {
                return Err(StackError::CoinOpsError(CoinOpsError::InvalidKindBytes(
                    kind_item.bytes().to_vec(),
                )))
            }
        };

        // Match the kind to a usize.
        match kind_as_usize {
            // Interpret as an account key.
            0 => {
                // Pop the account key.
                let account_key = stack_holder.pop()?;

                // Convert the account key to bytes.
                let account_key_bytes: [u8; 32] = match account_key.bytes().try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StackError::CoinOpsError(
                            CoinOpsError::InvalidAccountKeyBytes(account_key.bytes().to_vec()),
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
                        .ok_or(StackError::CoinOpsError(
                            CoinOpsError::UnableToGetAccountBalance(account_key_bytes),
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
            1 | 2 => {
                // Pop the contract id.
                let contract_id_bytes: [u8; 32] = match kind_as_usize {
                    // External contract id. Pop the contract id from the stack.
                    1 => {
                        // Pop the contract id from the stack.
                        let contract_id_stack_item = stack_holder.pop()?;

                        // Convert the contract id to bytes.
                        let contract_id_bytes: [u8; 32] =
                            match contract_id_stack_item.bytes().try_into() {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    return Err(StackError::CoinOpsError(
                                        CoinOpsError::InvalidContractIdBytes(
                                            contract_id_stack_item.bytes().to_vec(),
                                        ),
                                    ))
                                }
                            };

                        // Return the contract id as bytes.
                        contract_id_bytes
                    }
                    // Self contract id. Use the contract id from the stack holder.
                    2 => {
                        // Use the contract id from the stack holder.
                        let contract_id = stack_holder.contract_id();

                        // Return the contract id as bytes.
                        contract_id
                    }
                    // Invalid kind tier.
                    _ => {
                        return Err(StackError::CoinOpsError(CoinOpsError::InvalidKindTier(
                            kind_as_usize,
                        )))
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
                        .ok_or(StackError::CoinOpsError(
                            CoinOpsError::UnableToGetContractBalance(contract_id_bytes),
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
            _ => {
                return Err(StackError::CoinOpsError(CoinOpsError::InvalidKindTier(
                    kind_as_usize,
                )))
            }
        }

        Ok(())
    }

    /// Returns the bytecode for the `OP_BALANCE` opcode (0xc0).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc0]
    }
}
