use crate::{
    executive::stack::{
        stack_error::{CoinTransferError, StackError},
        stack_holder::StackHolder,
        stack_uint::{SafeConverter, StackItemUintExt},
    },
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Transfers coins from the contract into an account or to another contract.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_TRANSFER;

impl OP_TRANSFER {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the self contract id bytes.
        let self_contract_id_bytes = stack_holder.contract_id();

        // Pop the kind.
        let kind_item = stack_holder.pop()?;

        // Match the kind.
        match kind_item.is_false() {
            // Interpret as account key.
            true => {
                // Pop the account key.
                let account_key = stack_holder.pop()?;

                // Convert the account key to bytes.
                let destination_account_key_bytes: [u8; 32] = match account_key.bytes().try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StackError::CoinTransferError(
                            CoinTransferError::InvalidAccountKeyBytes(account_key.bytes().to_vec()),
                        ))
                    }
                };

                // Pop the amount.
                let amount = stack_holder.pop()?;

                // Convert the amount to a u64.
                let amount_as_stack_uint =
                    amount.to_stack_uint().ok_or(StackError::CoinTransferError(
                        CoinTransferError::InvalidAmountBytes(amount.bytes().to_vec()),
                    ))?;

                // Convert the amount to a u64.
                let amount_as_u64 =
                    amount_as_stack_uint
                        .to_u64()
                        .ok_or(StackError::CoinTransferError(
                            CoinTransferError::InvalidAmountBytes(amount.bytes().to_vec()),
                        ))?;

                // Get the account coin holder.
                let account_coin_holder = {
                    let _coin_holder = coin_holder.lock().await;
                    _coin_holder.account_coin_holder()
                };

                // Get the contract coin holder.
                let contract_coin_holder = {
                    let _coin_holder = coin_holder.lock().await;
                    _coin_holder.contract_coin_holder()
                };

                // Deduct from the self contract balance.
                {
                    let mut _contract_coin_holder = contract_coin_holder.lock().await;
                    _contract_coin_holder
                        .contract_balance_down(self_contract_id_bytes, amount_as_u64)
                        .map_err(|error| {
                            CoinTransferError::ContractBalanceDownError(
                                self_contract_id_bytes,
                                error,
                            )
                        })
                        .map_err(StackError::CoinTransferError)?;
                }

                // Add to the destination account balance.
                {
                    let mut _account_coin_holder = account_coin_holder.lock().await;
                    _account_coin_holder
                        .account_balance_up(destination_account_key_bytes, amount_as_u64)
                        .map_err(|error| {
                            CoinTransferError::AccountBalanceUpError(
                                destination_account_key_bytes,
                                error,
                            )
                        })
                        .map_err(StackError::CoinTransferError)?;
                }
            }
            // Interpret as contract id.
            false => {
                // Pop the contract id.
                let contract_id = stack_holder.pop()?;

                // Convert the contract id to bytes.
                let destination_contract_id_bytes: [u8; 32] = match contract_id.bytes().try_into() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return Err(StackError::CoinTransferError(
                            CoinTransferError::InvalidContractIdBytes(contract_id.bytes().to_vec()),
                        ))
                    }
                };

                // Pop the amount.
                let amount = stack_holder.pop()?;

                // Convert the amount to a u64.
                let amount_as_stack_uint =
                    amount.to_stack_uint().ok_or(StackError::CoinTransferError(
                        CoinTransferError::InvalidAmountBytes(amount.bytes().to_vec()),
                    ))?;

                // Convert the amount to a u64.
                let amount_as_u64 =
                    amount_as_stack_uint
                        .to_u64()
                        .ok_or(StackError::CoinTransferError(
                            CoinTransferError::InvalidAmountBytes(amount.bytes().to_vec()),
                        ))?;

                // Get the contract coin holder.
                let contract_coin_holder = {
                    let _coin_holder = coin_holder.lock().await;
                    _coin_holder.contract_coin_holder()
                };

                // Deduct from the self contract balance.
                {
                    let mut _contract_coin_holder = contract_coin_holder.lock().await;
                    _contract_coin_holder
                        .contract_balance_down(self_contract_id_bytes, amount_as_u64)
                        .map_err(|error| {
                            CoinTransferError::ContractBalanceDownError(
                                self_contract_id_bytes,
                                error,
                            )
                        })
                        .map_err(StackError::CoinTransferError)?;
                }

                // Add to the destination contract balance.
                {
                    let mut _contract_coin_holder = contract_coin_holder.lock().await;
                    _contract_coin_holder
                        .contract_balance_up(destination_contract_id_bytes, amount_as_u64)
                        .map_err(|error| {
                            CoinTransferError::ContractBalanceUpError(
                                destination_contract_id_bytes,
                                error,
                            )
                        })
                        .map_err(StackError::CoinTransferError)?;
                }
            }
        }

        Ok(())
    }

    /// Returns the bytecode for the `OP_TRANSFER` opcode (0xc2).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc2]
    }
}
