use super::{caller::Caller, exec_error::ExecutionError};
use crate::{
    executive::{
        opcode::{
            opcode::Opcode,
            opcodes::{
                altstack::{op_fromaltstack::OP_FROMALTSTACK, op_toaltstack::OP_TOALTSTACK},
                arithmetic::{
                    op_0notequal::OP_0NOTEQUAL, op_1add::OP_1ADD, op_1sub::OP_1SUB,
                    op_2div::OP_2DIV, op_2mul::OP_2MUL, op_add::OP_ADD, op_addmod::OP_ADDMOD,
                    op_booland::OP_BOOLAND, op_boolor::OP_BOOLOR, op_div::OP_DIV,
                    op_greaterthan::OP_GREATERTHAN, op_greaterthanorequal::OP_GREATERTHANOREQUAL,
                    op_lessthan::OP_LESSTHAN, op_lessthanorequal::OP_LESSTHANOREQUAL,
                    op_lshift::OP_LSHIFT, op_max::OP_MAX, op_min::OP_MIN, op_mul::OP_MUL,
                    op_mulmod::OP_MULMOD, op_not::OP_NOT, op_numequal::OP_NUMEQUAL,
                    op_numequalverify::OP_NUMEQUALVERIFY, op_numnotequal::OP_NUMNOTEQUAL,
                    op_rshift::OP_RSHIFT, op_sub::OP_SUB, op_within::OP_WITHIN,
                },
                bitwise::{
                    op_and::OP_AND, op_equal::OP_EQUAL, op_equalverify::OP_EQUALVERIFY,
                    op_invert::OP_INVERT, op_or::OP_OR, op_reverse::OP_REVERSE, op_xor::OP_XOR,
                },
                call::{op_call::OP_CALL, op_callext::OP_CALLEXT},
                callinfo::{
                    op_caller::OP_CALLER, op_opsbudget::OP_OPSBUDGET, op_opscounter::OP_OPSCOUNTER,
                    op_opsprice::OP_OPSPRICE, op_timestamp::OP_TIMESTAMP,
                },
                coin::{op_balance::OP_BALANCE, op_transfer::OP_TRANSFER},
                digest::{
                    op_blake2bvar::OP_BLAKE2BVAR, op_blake2svar::OP_BLAKE2SVAR,
                    op_hash160::OP_HASH160, op_hash256::OP_HASH256, op_ripemd160::OP_RIPEMD160,
                    op_sha1::OP_SHA1, op_sha256::OP_SHA256, op_taggedhash::OP_TAGGEDHASH,
                },
                flow::{
                    op_else::OP_ELSE, op_endif::OP_ENDIF, op_fail::OP_FAIL, op_if::OP_IF,
                    op_nop::OP_NOP, op_notif::OP_NOTIF, op_returnall::OP_RETURNALL,
                    op_returnerr::OP_RETURNERR, op_returnsome::OP_RETURNSOME, op_verify::OP_VERIFY,
                },
                memory::{op_free::OP_MFREE, op_mread::OP_MREAD, op_mwrite::OP_MWRITE},
                push::{
                    op_10::OP_10, op_11::OP_11, op_12::OP_12, op_13::OP_13, op_14::OP_14,
                    op_15::OP_15, op_16::OP_16, op_2::OP_2, op_3::OP_3, op_4::OP_4, op_5::OP_5,
                    op_6::OP_6, op_7::OP_7, op_8::OP_8, op_9::OP_9, op_false::OP_FALSE,
                    op_true::OP_TRUE,
                },
                secp::{
                    op_isinfinitesecppoint::OP_ISINFINITESECPPOINT,
                    op_iszerosecpscalar::OP_ISZEROSECPSCALAR,
                    op_pushsecpgeneratorpoint::OP_PUSHSECPGENERATORPOINT,
                    op_secppointadd::OP_SECPPOINTADD, op_secppointmul::OP_SECPPOINTMUL,
                    op_secpscalaradd::OP_SECPSCALARADD, op_secpscalarmul::OP_SECPSCALARMUL,
                },
                shadowing::{
                    op_shadow_alloc::OP_SHADOW_ALLOC, op_shadow_alloc_down::OP_SHADOW_ALLOC_DOWN,
                    op_shadow_alloc_down_all::OP_SHADOW_ALLOC_DOWN_ALL,
                    op_shadow_alloc_up::OP_SHADOW_ALLOC_UP,
                    op_shadow_alloc_up_all::OP_SHADOW_ALLOC_UP_ALL,
                },
                signature::{
                    op_checkblssig::OP_CHECKBLSSIG, op_checkblssigagg::OP_CHECKBLSSIGAGG,
                    op_checkschnorrsig::OP_CHECKSCHNORRSIG,
                    op_checkschnorrsigbip340::OP_CHECKSCHNORRSIGBIP340,
                },
                splice::{
                    op_cat::OP_CAT, op_left::OP_LEFT, op_right::OP_RIGHT, op_size::OP_SIZE,
                    op_split::OP_SPLIT,
                },
                stack::{
                    op_2drop::OP_2DROP, op_2dup::OP_2DUP, op_2over::OP_2OVER, op_2rot::OP_2ROT,
                    op_2swap::OP_2SWAP, op_3dup::OP_3DUP, op_depth::OP_DEPTH, op_drop::OP_DROP,
                    op_dup::OP_DUP, op_ifdup::OP_IFDUP, op_nip::OP_NIP, op_over::OP_OVER,
                    op_pick::OP_PICK, op_roll::OP_ROLL, op_rot::OP_ROT, op_swap::OP_SWAP,
                    op_tuck::OP_TUCK,
                },
                storage::{op_sread::OP_SREAD, op_swrite::OP_SWRITE},
            },
        },
        program::method::method_type::MethodType,
        stack::{stack_holder::StackHolder, stack_item::StackItem},
    },
    inscriptive::{
        coin_holder::coin_holder::COIN_HOLDER, repo::repo::PROGRAMS_REPO,
        state_holder::state_holder::STATE_HOLDER,
    },
};

/// The type of the external ops counter.
type ExternalOpsCounter = u32;

/// The type of the internal ops counter.
type InternalOpsCounter = u32;

/// The minimum satoshi payable allocation value.
pub const MIN_PAYABLE_ALLOCATION_VALUE: u32 = 10;

/// Executes a program method.
pub async fn execute(
    // Whether the execution is internal or external.
    internal: bool,
    // Caller can be the account itself or another contract.
    caller: Caller,
    // The contract id of the called contract.
    contract_id: [u8; 32],
    // The method index of the called contract.
    method_index: u8,
    // The stack items to be passed as arguments to the called contract.
    arg_values: Vec<StackItem>,
    // The timestamp.
    timestamp: u64,
    // The ops budget.
    ops_budget: u32,
    // The ops price.
    ops_price: u32,
    // The internal ops counter.
    internal_ops_counter: u32,
    // The external ops counter.
    external_ops_counter: ExternalOpsCounter,
    // The state holder.
    state_holder: &STATE_HOLDER,
    // The coin holder.
    coin_holder: &COIN_HOLDER,
    // The programs repo.
    programs_repo: &PROGRAMS_REPO,
) -> Result<(Vec<StackItem>, InternalOpsCounter, ExternalOpsCounter), ExecutionError> {
    // Get the program by contract id.
    let program = {
        let _programs_repo = programs_repo.lock().await;
        _programs_repo
            .program_by_contract_id(&contract_id)
            .ok_or(ExecutionError::ProgramNotFoundError(contract_id))?
    };

    // Get the program method by index.
    let program_method = match program.method_by_index(method_index) {
        Some(method) => method,
        None => return Err(ExecutionError::MethodNotFoundAtIndexError(method_index)),
    };

    // Match the method type.
    match program_method.method_type() {
        // Read only methods are considered a non-executable behavior.
        MethodType::ReadOnly => return Err(ExecutionError::ReadOnlyCallEncounteredError),

        // Internal methods are *valid* if its originated from the contract itself.
        // And *invalid* if originated from an external source.
        MethodType::Internal => {
            // Return an error if the call is not internal or the caller is an account.
            if !internal || caller.is_account() {
                return Err(ExecutionError::InvalidInternalCallError);
            }
        }

        // Callable methods are *valid* if originated from accounts or external contracts.
        // And *invalid* if originated internally from the contract itself.
        MethodType::Callable => {
            // Return an error if the call is internal.
            if internal {
                return Err(ExecutionError::InvalidInternalCallError);
            }
        }
    }

    // Match the args to the arg types.
    if !program_method.match_args(&arg_values) {
        return Err(ExecutionError::ArgTypeMismatchError);
    }

    // Get the payable allocation value.
    let payable_allocation_value = match program_method.payable_allocation_value(&arg_values) {
        Some(payable_allocation_value) => {
            // If a payable value is allocted it must be greater than MIN_PAYABLE_ALLOCATION.
            if payable_allocation_value < MIN_PAYABLE_ALLOCATION_VALUE {
                return Err(ExecutionError::MinPayableAllocationError);
            }

            // TODO: CHECK ENOUGH BALANCE.

            // If a payable value is allocted, the caller must also be an account.
            let _caller_key = match caller {
                Caller::Account(key) => key,
                Caller::Contract(_) => {
                    return Err(ExecutionError::PayableAllocationCallerIsNotAnAccountError);
                }
            };

            // If a payable value is allocted, this cannot be an internal call.
            if internal {
                return Err(ExecutionError::PayableWithInternalCallError);
            }

            // Insert the allocation into the accountant.
            //if let Err(error) = accountant.insert_alloc(caller_key, payable_allocation_value) {
            //    return Err(ExecutionError::AccountantAllocationInsertionError(error));
            //}

            payable_allocation_value
        }
        None => 0,
    };

    // Create a new stack holder.
    let mut stack_holder = match StackHolder::new_with_items(
        caller,
        contract_id,
        timestamp,
        payable_allocation_value,
        ops_budget,
        ops_price,
        internal_ops_counter,
        external_ops_counter,
        arg_values,
    ) {
        Ok(stack_holder) => stack_holder,
        Err(error) => return Err(ExecutionError::StackHolderInitializationError(error)),
    };

    // Execute the program method.
    for opcode in program_method.script().iter() {
        match opcode {
            // Data push opcodes.
            Opcode::OP_FALSE(OP_FALSE) => {
                OP_FALSE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_PUSHDATA(op_pushdata) => {
                op_pushdata
                    .execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_TRUE(OP_TRUE) => {
                OP_TRUE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2(OP_2) => {
                OP_2::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_3(OP_3) => {
                OP_3::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_4(OP_4) => {
                OP_4::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_5(OP_5) => {
                OP_5::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_6(OP_6) => {
                OP_6::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_7(OP_7) => {
                OP_7::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_8(OP_8) => {
                OP_8::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_9(OP_9) => {
                OP_9::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_10(OP_10) => {
                OP_10::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_11(OP_11) => {
                OP_11::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_12(OP_12) => {
                OP_12::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_13(OP_13) => {
                OP_13::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_14(OP_14) => {
                OP_14::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_15(OP_15) => {
                OP_15::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_16(OP_16) => {
                OP_16::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Flow control opcodes.
            Opcode::OP_NOP(_) => {
                OP_NOP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_RETURNERR(_) => {
                let error_item = OP_RETURNERR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;

                // Return the error item.
                return Err(ExecutionError::ReturnErrorFromStackError(error_item));
            }
            Opcode::OP_IF(_) => {
                OP_IF::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NOTIF(_) => {
                OP_NOTIF::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_RETURNALL(_) => {
                // If this is not an active execution, return immediately.
                if !stack_holder.active_execution() {
                    return Ok((vec![], internal_ops_counter, external_ops_counter));
                }

                // Return all items from the stack.
                let return_items = OP_RETURNALL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;

                // Get the ops spent.
                let internal_ops_counter = stack_holder.internal_ops_counter();

                // Get the up-to-date external ops counter.
                let new_external_ops_counter = stack_holder.external_ops_counter();

                // Return the items.
                return Ok((return_items, internal_ops_counter, new_external_ops_counter));
            }
            Opcode::OP_RETURNSOME(_) => {
                // If this is not an active execution, skip the opcode.
                if !stack_holder.active_execution() {
                    continue;
                }

                // Return some items from the stack.
                let return_items = OP_RETURNSOME::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;

                // Get the ops spent.
                let internal_ops_counter = stack_holder.internal_ops_counter();

                // Get the up-to-date external ops counter.
                let new_external_ops_counter = stack_holder.external_ops_counter();

                // Return the items.
                return Ok((return_items, internal_ops_counter, new_external_ops_counter));
            }
            Opcode::OP_ELSE(_) => {
                OP_ELSE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ENDIF(_) => {
                OP_ENDIF::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_VERIFY(_) => {
                OP_VERIFY::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_FAIL(_) => {
                OP_FAIL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Altstack operations.
            Opcode::OP_TOALTSTACK(_) => {
                OP_TOALTSTACK::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_FROMALTSTACK(_) => {
                OP_FROMALTSTACK::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Stack operations.
            Opcode::OP_2DROP(OP_2DROP) => {
                OP_2DROP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2DUP(OP_2DUP) => {
                OP_2DUP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_3DUP(OP_3DUP) => {
                OP_3DUP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2OVER(OP_2OVER) => {
                OP_2OVER::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2ROT(OP_2ROT) => {
                OP_2ROT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2SWAP(OP_2SWAP) => {
                OP_2SWAP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_IFDUP(OP_IFDUP) => {
                OP_IFDUP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_DEPTH(OP_DEPTH) => {
                OP_DEPTH::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_DROP(OP_DROP) => {
                OP_DROP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_DUP(OP_DUP) => {
                OP_DUP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NIP(OP_NIP) => {
                OP_NIP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_OVER(OP_OVER) => {
                OP_OVER::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_PICK(OP_PICK) => {
                OP_PICK::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ROLL(OP_ROLL) => {
                OP_ROLL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ROT(OP_ROT) => {
                OP_ROT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SWAP(OP_SWAP) => {
                OP_SWAP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_TUCK(OP_TUCK) => {
                OP_TUCK::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Splice opcodes.
            Opcode::OP_CAT(OP_CAT) => {
                OP_CAT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SPLIT(OP_SPLIT) => {
                OP_SPLIT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_LEFT(OP_LEFT) => {
                OP_LEFT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_RIGHT(OP_RIGHT) => {
                OP_RIGHT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SIZE(OP_SIZE) => {
                OP_SIZE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Bitwise opcodes.
            Opcode::OP_INVERT(OP_INVERT) => {
                OP_INVERT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_AND(OP_AND) => {
                OP_AND::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_OR(OP_OR) => {
                OP_OR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_XOR(OP_XOR) => {
                OP_XOR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_EQUAL(OP_EQUAL) => {
                OP_EQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_EQUALVERIFY(OP_EQUALVERIFY) => {
                OP_EQUALVERIFY::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_REVERSE(OP_REVERSE) => {
                OP_REVERSE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Arithmetic opcodes.
            Opcode::OP_1ADD(OP_1ADD) => {
                OP_1ADD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_1SUB(OP_1SUB) => {
                OP_1SUB::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2MUL(OP_2MUL) => {
                OP_2MUL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_2DIV(OP_2DIV) => {
                OP_2DIV::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ADDMOD(OP_ADDMOD) => {
                OP_ADDMOD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MULMOD(OP_MULMOD) => {
                OP_MULMOD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NOT(OP_NOT) => {
                OP_NOT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_0NOTEQUAL(OP_0NOTEQUAL) => {
                OP_0NOTEQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ADD(OP_ADD) => {
                OP_ADD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SUB(OP_SUB) => {
                OP_SUB::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MUL(OP_MUL) => {
                OP_MUL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_DIV(OP_DIV) => {
                OP_DIV::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_LSHIFT(OP_LSHIFT) => {
                OP_LSHIFT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_RSHIFT(OP_RSHIFT) => {
                OP_RSHIFT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_BOOLAND(OP_BOOLAND) => {
                OP_BOOLAND::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_BOOLOR(OP_BOOLOR) => {
                OP_BOOLOR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NUMEQUAL(OP_NUMEQUAL) => {
                OP_NUMEQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NUMEQUALVERIFY(OP_NUMEQUALVERIFY) => {
                OP_NUMEQUALVERIFY::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_NUMNOTEQUAL(OP_NUMNOTEQUAL) => {
                OP_NUMNOTEQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_LESSTHAN(OP_LESSTHAN) => {
                OP_LESSTHAN::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_GREATERTHAN(OP_GREATERTHAN) => {
                OP_GREATERTHAN::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_LESSTHANOREQUAL(OP_LESSTHANOREQUAL) => {
                OP_LESSTHANOREQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_GREATERTHANOREQUAL(OP_GREATERTHANOREQUAL) => {
                OP_GREATERTHANOREQUAL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MIN(OP_MIN) => {
                OP_MIN::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MAX(OP_MAX) => {
                OP_MAX::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_WITHIN(OP_WITHIN) => {
                OP_WITHIN::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Digest opcodes.
            Opcode::OP_RIPEMD160(OP_RIPEMD160) => {
                OP_RIPEMD160::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHA1(OP_SHA1) => {
                OP_SHA1::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHA256(OP_SHA256) => {
                OP_SHA256::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_HASH160(OP_HASH160) => {
                OP_HASH160::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_HASH256(OP_HASH256) => {
                OP_HASH256::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_TAGGEDHASH(OP_TAGGEDHASH) => {
                OP_TAGGEDHASH::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_BLAKE2BVAR(OP_BLAKE2BVAR) => {
                OP_BLAKE2BVAR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_BLAKE2SVAR(OP_BLAKE2SVAR) => {
                OP_BLAKE2SVAR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Secp opcodes.
            Opcode::OP_SECPSCALARADD(OP_SECPSCALARADD) => {
                OP_SECPSCALARADD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SECPSCALARMUL(OP_SECPSCALARMUL) => {
                OP_SECPSCALARMUL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SECPPOINTADD(OP_SECPPOINTADD) => {
                OP_SECPPOINTADD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SECPPOINTMUL(OP_SECPPOINTMUL) => {
                OP_SECPPOINTMUL::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_PUSHSECPGENERATORPOINT(OP_PUSHSECPGENERATORPOINT) => {
                OP_PUSHSECPGENERATORPOINT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ISZEROSECPSCALAR(OP_ISZEROSECPSCALAR) => {
                OP_ISZEROSECPSCALAR::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_ISINFINITESECPPOINT(OP_ISINFINITESECPPOINT) => {
                OP_ISINFINITESECPPOINT::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Digital signature opcodes.
            Opcode::OP_CHECKSCHNORRSIG(OP_CHECKSCHNORRSIG) => {
                OP_CHECKSCHNORRSIG::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_CHECKSCHNORRSIGBIP340(OP_CHECKSCHNORRSIGBIP340) => {
                OP_CHECKSCHNORRSIGBIP340::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_CHECKBLSSIG(OP_CHECKBLSSIG) => {
                OP_CHECKBLSSIG::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_CHECKBLSSIGAGG(OP_CHECKBLSSIGAGG) => {
                OP_CHECKBLSSIGAGG::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Call info opcodes.
            Opcode::OP_CALLER(OP_CALLER) => {
                OP_CALLER::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_OPSBUDGET(OP_OPSBUDGET) => {
                OP_OPSBUDGET::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_OPSCOUNTER(OP_OPSCOUNTER) => {
                OP_OPSCOUNTER::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_OPSPRICE(OP_OPSPRICE) => {
                OP_OPSPRICE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_TIMESTAMP(OP_TIMESTAMP) => {
                OP_TIMESTAMP::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Call opcodes.
            Opcode::OP_CALL(_) => {
                // If this is not an active execution, skip the opcode.
                if !stack_holder.active_execution() {
                    continue;
                }

                // Get the information about the internal call.
                let (method_index_to_be_called, call_arg_values) =
                    OP_CALL::execute(&mut stack_holder)
                        .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;

                // Call the internal contract.
                return Box::pin(execute(
                    true,        // Internal call.
                    caller,      // Caller remains unchanged for internal calls.
                    contract_id, // Contract ID is the same as the current contract id.
                    method_index_to_be_called,
                    call_arg_values,
                    timestamp,  // Timestamp is the same as the current timestamp.
                    ops_budget, // Ops budget is the same as the current ops budget.
                    ops_price,  // Ops price is the same as the current ops price.
                    stack_holder.internal_ops_counter(), // Remainder of the internal ops counter passed to the next call.
                    stack_holder.external_ops_counter(), // Remainder of the external ops counter passed to the next call.
                    state_holder,
                    coin_holder,
                    programs_repo,
                ))
                .await;
            }

            Opcode::OP_CALLEXT(_) => {
                // If this is not an active execution, skip the opcode.
                if !stack_holder.active_execution() {
                    continue;
                }

                // Get the information about the external call.
                let (contract_id_to_be_called, method_index_to_be_called, call_arg_values) =
                    OP_CALLEXT::execute(&mut stack_holder)
                        .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;

                // Raise and error if the same contract is being called as an external call.
                if contract_id_to_be_called == contract_id {
                    return Err(ExecutionError::ExternalCallAttemptAsInternalError);
                }

                // The caller for the next call is the current contract id.
                let caller = Caller::new_contract(contract_id);

                // Call the external contract.
                return Box::pin(execute(
                    false, // External call.
                    caller,
                    contract_id_to_be_called,
                    method_index_to_be_called,
                    call_arg_values,
                    timestamp,  // Timestamp is the same as the current timestamp.
                    ops_budget, // Ops budget is the same as the current ops budget.
                    ops_price,  // Ops price is the same as the current ops price.
                    stack_holder.internal_ops_counter(), // Remainder of the internal ops counter passed to the next call.
                    stack_holder.external_ops_counter(), // Remainder of the external ops counter passed to the next call.
                    state_holder,
                    coin_holder,
                    programs_repo,
                ))
                .await;
            }
            // Coin opcodes.
            Opcode::OP_BALANCE(OP_BALANCE) => {
                OP_BALANCE::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_TRANSFER(OP_TRANSFER) => {
                OP_TRANSFER::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Shadow space opcodes.
            Opcode::OP_SHADOW_ALLOC(OP_SHADOW_ALLOC) => {
                OP_SHADOW_ALLOC::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHADOW_ALLOC_UP(OP_SHADOW_ALLOC_UP) => {
                OP_SHADOW_ALLOC_UP::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHADOW_ALLOC_DOWN(OP_SHADOW_ALLOC_DOWN) => {
                OP_SHADOW_ALLOC_DOWN::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHADOW_ALLOC_UP_ALL(OP_SHADOW_ALLOC_UP_ALL) => {
                OP_SHADOW_ALLOC_UP_ALL::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SHADOW_ALLOC_DOWN_ALL(OP_SHADOW_ALLOC_DOWN_ALL) => {
                OP_SHADOW_ALLOC_DOWN_ALL::execute(&mut stack_holder, coin_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Storage opcodes.
            Opcode::OP_SWRITE(OP_SWRITE) => {
                OP_SWRITE::execute(&mut stack_holder, state_holder)
                    .await
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_SREAD(OP_SREAD) => {
                OP_SREAD::execute(&mut stack_holder, state_holder)
                    .await
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            // Memory opcodes.
            Opcode::OP_MWRITE(OP_MWRITE) => {
                OP_MWRITE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MREAD(OP_MREAD) => {
                OP_MREAD::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
            Opcode::OP_MFREE(OP_MFREE) => {
                OP_MFREE::execute(&mut stack_holder)
                    .map_err(|error| ExecutionError::OpcodeExecutionError(error))?;
            }
        }
    }

    return Err(ExecutionError::MethodNotReturnedAnyItemsError);
}
