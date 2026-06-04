// Integration test for the "Double your money contract" doubling contract.
//
// The contract has one callable, payable method, `double`, which:
//   - requires the sent amount X to satisfy 1 <= X <= 5000 (per-call cap)
//   - pays 2X back to the caller from the contract treasury (OP_TRANSFER)
//
// We deploy it on the Testbed chain, fund the treasury, and call the method
// directly through the execution engine, asserting the payout and the cap.

#[cfg(test)]
mod double_contract_tests {
    use cube::constructive::calldata::element_type::CalldataElementType;
    use cube::executive::executable::compiler::compiler::ProgramCompiler;
    use cube::executive::executable::executable::{Executable, Program};
    use cube::executive::executable::method::method_type::MethodType;
    use cube::executive::executable::method::program_method::ProgramMethod;
    use cube::executive::opcode::opcode::Opcode;
    use cube::executive::opcode::opcodes::arithmetic::op_2mul::OP_2MUL;
    use cube::executive::opcode::opcodes::arithmetic::op_within::OP_WITHIN;
    use cube::executive::opcode::opcodes::callinfo::op_caller::OP_CALLER;
    use cube::executive::opcode::opcodes::coin::op_transfer::OP_TRANSFER;
    use cube::executive::opcode::opcodes::flow::op_returnall::OP_RETURNALL;
    use cube::executive::opcode::opcodes::flow::op_verify::OP_VERIFY;
    use cube::executive::opcode::opcodes::push::op_pushdata::OP_PUSHDATA;
    use cube::executive::opcode::opcodes::push::op_true::OP_TRUE;
    use cube::executive::opcode::opcodes::stack::op_dup::OP_DUP;
    use cube::executive::stack::stack_item::StackItem;
    use cube::executive::stack::stack_uint::{StackItemUintExt, StackUint};
    use cube::executive::vm::program_execution::caller::Caller;
    use cube::executive::vm::program_execution::exec::execute;
    use cube::inscriptive::coin_manager::coin_manager::{
        erase_coin_manager, CoinManager, COIN_MANAGER,
    };
    use cube::inscriptive::registery::registery::{erase_registery, Registery, REGISTERY};
    use cube::inscriptive::state_manager::state_manager::{
        erase_state_manager, StateManager, STATE_MANAGER,
    };
    use cube::operative::run_args::chain::Chain;

    // Builds the `double` method's opcode script.
    fn double_script() -> Vec<Opcode> {
        // Minimal stack-uint encoding of the 5000-sat cap.
        let cap_item = StackItem::from_stack_uint(StackUint::from(5000u64));

        vec![
            // Stack starts as [X] (the payable arg).
            Opcode::OP_DUP(OP_DUP),                                  // [X, X]
            Opcode::OP_TRUE(OP_TRUE),                                // [X, X, 1]  (min)
            Opcode::OP_PUSHDATA(OP_PUSHDATA(cap_item.bytes().to_vec())), // [X, X, 1, 5000] (max)
            Opcode::OP_WITHIN(OP_WITHIN),                            // [X, (1<=X<=5000)]
            Opcode::OP_VERIFY(OP_VERIFY),                            // [X]  (abort if X>5000)
            Opcode::OP_2MUL(OP_2MUL),                                // [2X, ok] (2MUL pushes result + success flag)
            Opcode::OP_VERIFY(OP_VERIFY),                            // [2X] (consume flag; abort on overflow)
            Opcode::OP_CALLER(OP_CALLER),                            // [2X, caller_key, false]
            Opcode::OP_TRANSFER(OP_TRANSFER),                        // pay caller 2X from treasury
            Opcode::OP_RETURNALL(OP_RETURNALL),                      // clean return
        ]
    }

    fn build_program() -> Program {
        let method = ProgramMethod::new(
            "double".to_string(),
            MethodType::Callable,
            vec![CalldataElementType::Payable],
            double_script(),
        )
        .expect("failed to build method");

        Executable::new("Double your money contract".to_string(), None, vec![method])
            .expect("failed to build program")
    }

    async fn run_double(payable_x: u64, treasury: u64) -> Result<(u64, u64), String> {
        let chain = Chain::Testbed;

        // Fresh managers.
        erase_registery(chain);
        let registery: REGISTERY = Registery::new(chain).expect("registery");
        erase_coin_manager(chain);
        let coin_manager: COIN_MANAGER = CoinManager::new(chain).expect("coin manager");
        erase_state_manager(chain);
        let state_manager: STATE_MANAGER = StateManager::new(chain).expect("state manager");

        // Round-trip the program through the codec (compile -> decompile), exactly
        // as `deploy` does. This catches opcode bytecode/decompiler mismatches that
        // a direct typed-enum execution would miss.
        let program = {
            let compiled = build_program().compile().expect("compile");
            let mut stream = compiled.into_iter();
            Program::decompile(&mut stream).expect("decompile")
        };
        assert_eq!(
            program,
            build_program(),
            "program must survive the compile/decompile round-trip unchanged"
        );
        let contract_id = program.contract_id();
        let caller_key: [u8; 32] = [0x11u8; 32];
        let timestamp = 1_780_000_000u64;

        // Register + fund the contract treasury.
        {
            let mut r = registery.lock().await;
            r.register_contract(contract_id, timestamp, program.clone())
                .map_err(|e| format!("registery.register_contract: {:?}", e))?;
        }
        {
            let mut c = coin_manager.lock().await;
            c.register_contract(contract_id, treasury)
                .map_err(|e| format!("coin.register_contract: {:?}", e))?;
            // Register the caller account with zero balance.
            c.register_account(caller_key, 0)
                .map_err(|e| format!("coin.register_account: {:?}", e))?;
        }
        {
            let mut r = registery.lock().await;
            r.register_account(caller_key, timestamp, None, None, None, None)
                .map_err(|e| format!("registery.register_account: {:?}", e))?;
        }

        // Commit registrations from the ephemeral delta to permanent storage,
        // so the execution engine can find the contract.
        {
            let mut r = registery.lock().await;
            r.apply_changes()
                .map_err(|e| format!("registery.apply_changes: {:?}", e))?;
        }
        {
            let mut c = coin_manager.lock().await;
            c.apply_changes()
                .map_err(|e| format!("coin.apply_changes: {:?}", e))?;
        }

        // Call double(X).
        let arg_values = vec![StackItem::from_stack_uint(StackUint::from(payable_x))];
        execute(
            false,
            Caller::Account(caller_key),
            contract_id,
            0, // method index
            arg_values,
            timestamp,
            1_000_000, // ops budget
            0,         // ops price
            0,
            0,
            &state_manager,
            &coin_manager,
            &registery,
        )
        .await
        .map_err(|e| format!("execute: {:?}", e))?;

        // Commit the transfer the method performed (delta -> permanent), then read.
        {
            let mut c = coin_manager.lock().await;
            c.apply_changes()
                .map_err(|e| format!("coin.apply_changes (post-exec): {:?}", e))?;
        }

        // Read resulting balances.
        let c = coin_manager.lock().await;
        let caller_bal = c.get_account_balance(caller_key).unwrap_or(0);
        let contract_bal = c.get_contract_balance(contract_id).unwrap_or(0);
        Ok((caller_bal, contract_bal))
    }

    #[test]
    fn print_program_bytes() {
        use cube::executive::executable::compiler::compiler::ProgramCompiler;
        let program = build_program();
        let bytes = program.compile().expect("compile");
        println!("PROGRAM_BYTES=0x{}", hex::encode(bytes));
        println!("CONTRACT_ID=0x{}", hex::encode(program.contract_id()));
    }

    #[tokio::test]
    async fn double_pays_2x_within_cap() {
        // Send 5000, expect 10000 back; treasury 50000 -> 40000.
        let (caller_bal, contract_bal) = run_double(5000, 50_000).await.expect("should succeed");
        assert_eq!(caller_bal, 10_000, "caller should receive 2X");
        assert_eq!(contract_bal, 40_000, "treasury should drop by 2X");
    }

    #[tokio::test]
    async fn double_rejects_over_cap() {
        // 5001 exceeds the 5000 cap -> OP_VERIFY fails -> execute errors.
        let result = run_double(5001, 50_000).await;
        assert!(result.is_err(), "over-cap call must fail, got {:?}", result);
    }
}
