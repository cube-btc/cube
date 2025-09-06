use super::compiler_error::{OpcodeCompileError, OpcodeDecompileError};
use crate::executive::opcode::opcode::Opcode;
use crate::executive::opcode::opcodes::altstack::op_fromaltstack::OP_FROMALTSTACK;
use crate::executive::opcode::opcodes::altstack::op_toaltstack::OP_TOALTSTACK;
use crate::executive::opcode::opcodes::arithmetic::op_0notequal::OP_0NOTEQUAL;
use crate::executive::opcode::opcodes::arithmetic::op_1add::OP_1ADD;
use crate::executive::opcode::opcodes::arithmetic::op_1sub::OP_1SUB;
use crate::executive::opcode::opcodes::arithmetic::op_2div::OP_2DIV;
use crate::executive::opcode::opcodes::arithmetic::op_2mul::OP_2MUL;
use crate::executive::opcode::opcodes::arithmetic::op_add::OP_ADD;
use crate::executive::opcode::opcodes::arithmetic::op_addmod::OP_ADDMOD;
use crate::executive::opcode::opcodes::arithmetic::op_booland::OP_BOOLAND;
use crate::executive::opcode::opcodes::arithmetic::op_boolor::OP_BOOLOR;
use crate::executive::opcode::opcodes::arithmetic::op_div::OP_DIV;
use crate::executive::opcode::opcodes::arithmetic::op_greaterthan::OP_GREATERTHAN;
use crate::executive::opcode::opcodes::arithmetic::op_greaterthanorequal::OP_GREATERTHANOREQUAL;
use crate::executive::opcode::opcodes::arithmetic::op_lessthan::OP_LESSTHAN;
use crate::executive::opcode::opcodes::arithmetic::op_lessthanorequal::OP_LESSTHANOREQUAL;
use crate::executive::opcode::opcodes::arithmetic::op_lshift::OP_LSHIFT;
use crate::executive::opcode::opcodes::arithmetic::op_max::OP_MAX;
use crate::executive::opcode::opcodes::arithmetic::op_min::OP_MIN;
use crate::executive::opcode::opcodes::arithmetic::op_mul::OP_MUL;
use crate::executive::opcode::opcodes::arithmetic::op_mulmod::OP_MULMOD;
use crate::executive::opcode::opcodes::arithmetic::op_not::OP_NOT;
use crate::executive::opcode::opcodes::arithmetic::op_numequal::OP_NUMEQUAL;
use crate::executive::opcode::opcodes::arithmetic::op_numequalverify::OP_NUMEQUALVERIFY;
use crate::executive::opcode::opcodes::arithmetic::op_numnotequal::OP_NUMNOTEQUAL;
use crate::executive::opcode::opcodes::arithmetic::op_rshift::OP_RSHIFT;
use crate::executive::opcode::opcodes::arithmetic::op_sub::OP_SUB;
use crate::executive::opcode::opcodes::arithmetic::op_within::OP_WITHIN;
use crate::executive::opcode::opcodes::bitwise::op_and::OP_AND;
use crate::executive::opcode::opcodes::bitwise::op_equal::OP_EQUAL;
use crate::executive::opcode::opcodes::bitwise::op_equalverify::OP_EQUALVERIFY;
use crate::executive::opcode::opcodes::bitwise::op_invert::OP_INVERT;
use crate::executive::opcode::opcodes::bitwise::op_or::OP_OR;
use crate::executive::opcode::opcodes::bitwise::op_reverse::OP_REVERSE;
use crate::executive::opcode::opcodes::bitwise::op_xor::OP_XOR;
use crate::executive::opcode::opcodes::call::op_call::OP_CALL;
use crate::executive::opcode::opcodes::call::op_callext::OP_CALLEXT;
use crate::executive::opcode::opcodes::callinfo::op_caller::OP_CALLER;
use crate::executive::opcode::opcodes::callinfo::op_opsbudget::OP_OPSBUDGET;
use crate::executive::opcode::opcodes::callinfo::op_opscounter::OP_OPSCOUNTER;
use crate::executive::opcode::opcodes::callinfo::op_opsprice::OP_OPSPRICE;
use crate::executive::opcode::opcodes::callinfo::op_timestamp::OP_TIMESTAMP;
use crate::executive::opcode::opcodes::coin::op_ext_balance::OP_EXT_BALANCE;
use crate::executive::opcode::opcodes::coin::op_self_balance::OP_SELF_BALANCE;
use crate::executive::opcode::opcodes::coin::op_transfer::OP_TRANSFER;
use crate::executive::opcode::opcodes::digest::op_blake2bvar::OP_BLAKE2BVAR;
use crate::executive::opcode::opcodes::digest::op_blake2svar::OP_BLAKE2SVAR;
use crate::executive::opcode::opcodes::digest::op_hash160::OP_HASH160;
use crate::executive::opcode::opcodes::digest::op_hash256::OP_HASH256;
use crate::executive::opcode::opcodes::digest::op_ripemd160::OP_RIPEMD160;
use crate::executive::opcode::opcodes::digest::op_sha1::OP_SHA1;
use crate::executive::opcode::opcodes::digest::op_sha256::OP_SHA256;
use crate::executive::opcode::opcodes::digest::op_taggedhash::OP_TAGGEDHASH;
use crate::executive::opcode::opcodes::flow::op_else::OP_ELSE;
use crate::executive::opcode::opcodes::flow::op_endif::OP_ENDIF;
use crate::executive::opcode::opcodes::flow::op_fail::OP_FAIL;
use crate::executive::opcode::opcodes::flow::op_if::OP_IF;
use crate::executive::opcode::opcodes::flow::op_jump::OP_JUMP;
use crate::executive::opcode::opcodes::flow::op_nop::OP_NOP;
use crate::executive::opcode::opcodes::flow::op_notif::OP_NOTIF;
use crate::executive::opcode::opcodes::flow::op_returnall::OP_RETURNALL;
use crate::executive::opcode::opcodes::flow::op_returnsome::OP_RETURNSOME;
use crate::executive::opcode::opcodes::flow::op_verify::OP_VERIFY;
use crate::executive::opcode::opcodes::memory::op_free::OP_MFREE;
use crate::executive::opcode::opcodes::memory::op_mread::OP_MREAD;
use crate::executive::opcode::opcodes::memory::op_mwrite::OP_MWRITE;
use crate::executive::opcode::opcodes::push::op_10::OP_10;
use crate::executive::opcode::opcodes::push::op_11::OP_11;
use crate::executive::opcode::opcodes::push::op_12::OP_12;
use crate::executive::opcode::opcodes::push::op_13::OP_13;
use crate::executive::opcode::opcodes::push::op_14::OP_14;
use crate::executive::opcode::opcodes::push::op_15::OP_15;
use crate::executive::opcode::opcodes::push::op_16::OP_16;
use crate::executive::opcode::opcodes::push::op_2::OP_2;
use crate::executive::opcode::opcodes::push::op_3::OP_3;
use crate::executive::opcode::opcodes::push::op_4::OP_4;
use crate::executive::opcode::opcodes::push::op_5::OP_5;
use crate::executive::opcode::opcodes::push::op_6::OP_6;
use crate::executive::opcode::opcodes::push::op_7::OP_7;
use crate::executive::opcode::opcodes::push::op_8::OP_8;
use crate::executive::opcode::opcodes::push::op_9::OP_9;
use crate::executive::opcode::opcodes::push::op_false::OP_FALSE;
use crate::executive::opcode::opcodes::push::op_pushdata::OP_PUSHDATA;
use crate::executive::opcode::opcodes::push::op_true::OP_TRUE;
use crate::executive::opcode::opcodes::secp::op_isinfinitesecppoint::OP_ISINFINITESECPPOINT;
use crate::executive::opcode::opcodes::secp::op_iszerosecpscalar::OP_ISZEROSECPSCALAR;
use crate::executive::opcode::opcodes::secp::op_pushsecpgeneratorpoint::OP_PUSHSECPGENERATORPOINT;
use crate::executive::opcode::opcodes::secp::op_secppointadd::OP_SECPPOINTADD;
use crate::executive::opcode::opcodes::secp::op_secppointmul::OP_SECPPOINTMUL;
use crate::executive::opcode::opcodes::secp::op_secpscalaradd::OP_SECPSCALARADD;
use crate::executive::opcode::opcodes::secp::op_secpscalarmul::OP_SECPSCALARMUL;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc::OP_SHADOW_ALLOC;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc_down::OP_SHADOW_ALLOC_DOWN;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc_down_all::OP_SHADOW_ALLOC_DOWN_ALL;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc_up::OP_SHADOW_ALLOC_UP;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc_up_all::OP_SHADOW_ALLOC_UP_ALL;
use crate::executive::opcode::opcodes::shadowing::op_shadow_alloc_val::OP_SHADOW_ALLOC_VAL;
use crate::executive::opcode::opcodes::shadowing::op_shadow_allocs_sum::OP_SHADOW_ALLOCS_SUM;
use crate::executive::opcode::opcodes::shadowing::op_shadow_dealloc::OP_SHADOW_DEALLOC;
use crate::executive::opcode::opcodes::shadowing::op_shadow_has_alloc::OP_SHADOW_HAS_ALLOC;
use crate::executive::opcode::opcodes::shadowing::op_shadow_num_allocs::OP_SHADOW_NUM_ALLOCS;
use crate::executive::opcode::opcodes::signature::op_checkblssig::OP_CHECKBLSSIG;
use crate::executive::opcode::opcodes::signature::op_checkblssigagg::OP_CHECKBLSSIGAGG;
use crate::executive::opcode::opcodes::signature::op_checkschnorrsig::OP_CHECKSCHNORRSIG;
use crate::executive::opcode::opcodes::signature::op_checkschnorrsigbip340::OP_CHECKSCHNORRSIGBIP340;
use crate::executive::opcode::opcodes::splice::op_cat::OP_CAT;
use crate::executive::opcode::opcodes::splice::op_left::OP_LEFT;
use crate::executive::opcode::opcodes::splice::op_right::OP_RIGHT;
use crate::executive::opcode::opcodes::splice::op_size::OP_SIZE;
use crate::executive::opcode::opcodes::splice::op_split::OP_SPLIT;
use crate::executive::opcode::opcodes::stack::op_2drop::OP_2DROP;
use crate::executive::opcode::opcodes::stack::op_2dup::OP_2DUP;
use crate::executive::opcode::opcodes::stack::op_2over::OP_2OVER;
use crate::executive::opcode::opcodes::stack::op_2rot::OP_2ROT;
use crate::executive::opcode::opcodes::stack::op_2swap::OP_2SWAP;
use crate::executive::opcode::opcodes::stack::op_3dup::OP_3DUP;
use crate::executive::opcode::opcodes::stack::op_depth::OP_DEPTH;
use crate::executive::opcode::opcodes::stack::op_drop::OP_DROP;
use crate::executive::opcode::opcodes::stack::op_dup::OP_DUP;
use crate::executive::opcode::opcodes::stack::op_ifdup::OP_IFDUP;
use crate::executive::opcode::opcodes::stack::op_nip::OP_NIP;
use crate::executive::opcode::opcodes::stack::op_over::OP_OVER;
use crate::executive::opcode::opcodes::stack::op_pick::OP_PICK;
use crate::executive::opcode::opcodes::stack::op_roll::OP_ROLL;
use crate::executive::opcode::opcodes::stack::op_rot::OP_ROT;
use crate::executive::opcode::opcodes::stack::op_swap::OP_SWAP;
use crate::executive::opcode::opcodes::stack::op_tuck::OP_TUCK;
use crate::executive::opcode::opcodes::storage::op_sread::OP_SREAD;
use crate::executive::opcode::opcodes::storage::op_swrite::OP_SWRITE;

/// A trait for compiling and decompiling an opcode.
pub trait OpcodeCompiler {
    /// Compiles the opcode into a bytecode.
    fn compile(&self) -> Result<Vec<u8>, OpcodeCompileError>;
    /// Decompiles an opcode from a bytecode stream.
    fn decompile<I>(bytecode_stream: &mut I) -> Result<Opcode, OpcodeDecompileError>
    where
        I: Iterator<Item = u8>;
}

impl OpcodeCompiler for Opcode {
    fn compile(&self) -> Result<Vec<u8>, OpcodeCompileError> {
        match self {
            // Data push
            Opcode::OP_FALSE(_) => Ok(OP_FALSE::bytecode()),
            Opcode::OP_PUSHDATA(op_pushdata) => op_pushdata
                .compiled_bytes()
                .map(Ok)
                .unwrap_or_else(|| Err(OpcodeCompileError::InvalidPushDataLength)),
            Opcode::OP_TRUE(_) => Ok(OP_TRUE::bytecode()),
            Opcode::OP_2(_) => Ok(OP_2::bytecode()),
            Opcode::OP_3(_) => Ok(OP_3::bytecode()),
            Opcode::OP_4(_) => Ok(OP_4::bytecode()),
            Opcode::OP_5(_) => Ok(OP_5::bytecode()),
            Opcode::OP_6(_) => Ok(OP_6::bytecode()),
            Opcode::OP_7(_) => Ok(OP_7::bytecode()),
            Opcode::OP_8(_) => Ok(OP_8::bytecode()),
            Opcode::OP_9(_) => Ok(OP_9::bytecode()),
            Opcode::OP_10(_) => Ok(OP_10::bytecode()),
            Opcode::OP_11(_) => Ok(OP_11::bytecode()),
            Opcode::OP_12(_) => Ok(OP_12::bytecode()),
            Opcode::OP_13(_) => Ok(OP_13::bytecode()),
            Opcode::OP_14(_) => Ok(OP_14::bytecode()),
            Opcode::OP_15(_) => Ok(OP_15::bytecode()),
            Opcode::OP_16(_) => Ok(OP_16::bytecode()),

            // Flow control
            Opcode::OP_NOP(_) => Ok(OP_NOP::bytecode()),
            Opcode::OP_JUMP(_) => Ok(OP_JUMP::bytecode()),
            Opcode::OP_IF(_) => Ok(OP_IF::bytecode()),
            Opcode::OP_NOTIF(_) => Ok(OP_NOTIF::bytecode()),
            Opcode::OP_RETURNALL(_) => Ok(OP_RETURNALL::bytecode()),
            Opcode::OP_RETURNSOME(_) => Ok(OP_RETURNSOME::bytecode()),
            Opcode::OP_ELSE(_) => Ok(OP_ELSE::bytecode()),
            Opcode::OP_ENDIF(_) => Ok(OP_ENDIF::bytecode()),
            Opcode::OP_VERIFY(_) => Ok(OP_VERIFY::bytecode()),
            Opcode::OP_FAIL(_) => Ok(OP_FAIL::bytecode()),

            // Altstack
            Opcode::OP_TOALTSTACK(_) => Ok(OP_TOALTSTACK::bytecode()),
            Opcode::OP_FROMALTSTACK(_) => Ok(OP_FROMALTSTACK::bytecode()),

            // Stack
            Opcode::OP_IFDUP(_) => Ok(OP_IFDUP::bytecode()),
            Opcode::OP_DEPTH(_) => Ok(OP_DEPTH::bytecode()),
            Opcode::OP_DROP(_) => Ok(OP_DROP::bytecode()),
            Opcode::OP_DUP(_) => Ok(OP_DUP::bytecode()),
            Opcode::OP_NIP(_) => Ok(OP_NIP::bytecode()),
            Opcode::OP_OVER(_) => Ok(OP_OVER::bytecode()),
            Opcode::OP_PICK(_) => Ok(OP_PICK::bytecode()),
            Opcode::OP_ROLL(_) => Ok(OP_ROLL::bytecode()),
            Opcode::OP_ROT(_) => Ok(OP_ROT::bytecode()),
            Opcode::OP_SWAP(_) => Ok(OP_SWAP::bytecode()),
            Opcode::OP_TUCK(_) => Ok(OP_TUCK::bytecode()),
            Opcode::OP_2DROP(_) => Ok(OP_2DROP::bytecode()),
            Opcode::OP_2DUP(_) => Ok(OP_2DUP::bytecode()),
            Opcode::OP_3DUP(_) => Ok(OP_3DUP::bytecode()),
            Opcode::OP_2OVER(_) => Ok(OP_2OVER::bytecode()),
            Opcode::OP_2ROT(_) => Ok(OP_2ROT::bytecode()),
            Opcode::OP_2SWAP(_) => Ok(OP_2SWAP::bytecode()),

            // Splice
            Opcode::OP_CAT(_) => Ok(OP_CAT::bytecode()),
            Opcode::OP_SPLIT(_) => Ok(OP_SPLIT::bytecode()),
            Opcode::OP_LEFT(_) => Ok(OP_LEFT::bytecode()),
            Opcode::OP_RIGHT(_) => Ok(OP_RIGHT::bytecode()),
            Opcode::OP_SIZE(_) => Ok(OP_SIZE::bytecode()),

            // Bitwise
            Opcode::OP_INVERT(_) => Ok(OP_INVERT::bytecode()),
            Opcode::OP_AND(_) => Ok(OP_AND::bytecode()),
            Opcode::OP_OR(_) => Ok(OP_OR::bytecode()),
            Opcode::OP_XOR(_) => Ok(OP_XOR::bytecode()),
            Opcode::OP_EQUAL(_) => Ok(OP_EQUAL::bytecode()),
            Opcode::OP_EQUALVERIFY(_) => Ok(OP_EQUALVERIFY::bytecode()),
            Opcode::OP_REVERSE(_) => Ok(OP_REVERSE::bytecode()),

            // Arithmetic
            Opcode::OP_1ADD(_) => Ok(OP_1ADD::bytecode()),
            Opcode::OP_1SUB(_) => Ok(OP_1SUB::bytecode()),
            Opcode::OP_2MUL(_) => Ok(OP_2MUL::bytecode()),
            Opcode::OP_2DIV(_) => Ok(OP_2DIV::bytecode()),
            Opcode::OP_ADDMOD(_) => Ok(OP_ADDMOD::bytecode()),
            Opcode::OP_MULMOD(_) => Ok(OP_MULMOD::bytecode()),
            Opcode::OP_NOT(_) => Ok(OP_NOT::bytecode()),
            Opcode::OP_0NOTEQUAL(_) => Ok(OP_0NOTEQUAL::bytecode()),
            Opcode::OP_ADD(_) => Ok(OP_ADD::bytecode()),
            Opcode::OP_SUB(_) => Ok(OP_SUB::bytecode()),
            Opcode::OP_MUL(_) => Ok(OP_MUL::bytecode()),
            Opcode::OP_DIV(_) => Ok(OP_DIV::bytecode()),
            Opcode::OP_LSHIFT(_) => Ok(OP_LSHIFT::bytecode()),
            Opcode::OP_RSHIFT(_) => Ok(OP_RSHIFT::bytecode()),
            Opcode::OP_BOOLAND(_) => Ok(OP_BOOLAND::bytecode()),
            Opcode::OP_BOOLOR(_) => Ok(OP_BOOLOR::bytecode()),
            Opcode::OP_NUMEQUAL(_) => Ok(OP_NUMEQUAL::bytecode()),
            Opcode::OP_NUMEQUALVERIFY(_) => Ok(OP_NUMEQUALVERIFY::bytecode()),
            Opcode::OP_NUMNOTEQUAL(_) => Ok(OP_NUMNOTEQUAL::bytecode()),
            Opcode::OP_LESSTHAN(_) => Ok(OP_LESSTHAN::bytecode()),
            Opcode::OP_GREATERTHAN(_) => Ok(OP_GREATERTHAN::bytecode()),
            Opcode::OP_LESSTHANOREQUAL(_) => Ok(OP_LESSTHANOREQUAL::bytecode()),
            Opcode::OP_GREATERTHANOREQUAL(_) => Ok(OP_GREATERTHANOREQUAL::bytecode()),
            Opcode::OP_MIN(_) => Ok(OP_MIN::bytecode()),
            Opcode::OP_MAX(_) => Ok(OP_MAX::bytecode()),
            Opcode::OP_WITHIN(_) => Ok(OP_WITHIN::bytecode()),

            // Hashing
            Opcode::OP_RIPEMD160(_) => Ok(OP_RIPEMD160::bytecode()),
            Opcode::OP_SHA1(_) => Ok(OP_SHA1::bytecode()),
            Opcode::OP_SHA256(_) => Ok(OP_SHA256::bytecode()),
            Opcode::OP_HASH160(_) => Ok(OP_HASH160::bytecode()),
            Opcode::OP_HASH256(_) => Ok(OP_HASH256::bytecode()),
            Opcode::OP_TAGGEDHASH(_) => Ok(OP_TAGGEDHASH::bytecode()),
            Opcode::OP_BLAKE2BVAR(_) => Ok(OP_BLAKE2BVAR::bytecode()),
            Opcode::OP_BLAKE2SVAR(_) => Ok(OP_BLAKE2SVAR::bytecode()),

            // Secp
            Opcode::OP_SECPSCALARADD(_) => Ok(OP_SECPSCALARADD::bytecode()),
            Opcode::OP_SECPSCALARMUL(_) => Ok(OP_SECPSCALARMUL::bytecode()),
            Opcode::OP_SECPPOINTADD(_) => Ok(OP_SECPPOINTADD::bytecode()),
            Opcode::OP_SECPPOINTMUL(_) => Ok(OP_SECPPOINTMUL::bytecode()),
            Opcode::OP_PUSHSECPGENERATORPOINT(_) => Ok(OP_PUSHSECPGENERATORPOINT::bytecode()),
            Opcode::OP_ISZEROSECPSCALAR(_) => Ok(OP_ISZEROSECPSCALAR::bytecode()),
            Opcode::OP_ISINFINITESECPPOINT(_) => Ok(OP_ISINFINITESECPPOINT::bytecode()),

            // Digital signatures
            Opcode::OP_CHECKSCHNORRSIG(_) => Ok(OP_CHECKSCHNORRSIG::bytecode()),
            Opcode::OP_CHECKSCHNORRSIGBIP340(_) => Ok(OP_CHECKSCHNORRSIGBIP340::bytecode()),
            Opcode::OP_CHECKBLSSIG(_) => Ok(OP_CHECKBLSSIG::bytecode()),
            Opcode::OP_CHECKBLSSIGAGG(_) => Ok(OP_CHECKBLSSIGAGG::bytecode()),

            // Call info
            Opcode::OP_CALLER(_) => Ok(OP_CALLER::bytecode()),
            Opcode::OP_OPSBUDGET(_) => Ok(OP_OPSBUDGET::bytecode()),
            Opcode::OP_OPSCOUNTER(_) => Ok(OP_OPSCOUNTER::bytecode()),
            Opcode::OP_OPSPRICE(_) => Ok(OP_OPSPRICE::bytecode()),
            Opcode::OP_TIMESTAMP(_) => Ok(OP_TIMESTAMP::bytecode()),

            // Call
            Opcode::OP_CALL(_) => Ok(OP_CALL::bytecode()),
            Opcode::OP_CALLEXT(_) => Ok(OP_CALLEXT::bytecode()),

            // Shadowing
            Opcode::OP_SHADOW_HAS_ALLOC(_) => Ok(OP_SHADOW_HAS_ALLOC::bytecode()),
            Opcode::OP_SHADOW_DEALLOC(_) => Ok(OP_SHADOW_DEALLOC::bytecode()),
            Opcode::OP_SHADOW_ALLOC_VAL(_) => Ok(OP_SHADOW_ALLOC_VAL::bytecode()),
            Opcode::OP_SHADOW_ALLOC(_) => Ok(OP_SHADOW_ALLOC::bytecode()),
            Opcode::OP_SHADOW_ALLOC_UP(_) => Ok(OP_SHADOW_ALLOC_UP::bytecode()),
            Opcode::OP_SHADOW_ALLOC_DOWN(_) => Ok(OP_SHADOW_ALLOC_DOWN::bytecode()),
            Opcode::OP_SHADOW_ALLOC_UP_ALL(_) => Ok(OP_SHADOW_ALLOC_UP_ALL::bytecode()),
            Opcode::OP_SHADOW_ALLOC_DOWN_ALL(_) => Ok(OP_SHADOW_ALLOC_DOWN_ALL::bytecode()),
            Opcode::OP_SHADOW_NUM_ALLOCS(_) => Ok(OP_SHADOW_NUM_ALLOCS::bytecode()),
            Opcode::OP_SHADOW_ALLOCS_SUM(_) => Ok(OP_SHADOW_ALLOCS_SUM::bytecode()),

            // Coin
            Opcode::OP_EXT_BALANCE(_) => Ok(OP_EXT_BALANCE::bytecode()),
            Opcode::OP_SELF_BALANCE(_) => Ok(OP_SELF_BALANCE::bytecode()),
            Opcode::OP_TRANSFER(_) => Ok(OP_TRANSFER::bytecode()),

            // Storage
            Opcode::OP_SWRITE(_) => Ok(OP_SWRITE::bytecode()),
            Opcode::OP_SREAD(_) => Ok(OP_SREAD::bytecode()),

            // Memory
            Opcode::OP_MWRITE(_) => Ok(OP_MWRITE::bytecode()),
            Opcode::OP_MREAD(_) => Ok(OP_MREAD::bytecode()),
            Opcode::OP_MFREE(_) => Ok(OP_MFREE::bytecode()),
        }
    }

    fn decompile<I>(bytecode_stream: &mut I) -> Result<Opcode, OpcodeDecompileError>
    where
        I: Iterator<Item = u8>,
    {
        // Collect one byte from the bytecode stream.
        let byte = bytecode_stream
            .next()
            .ok_or(OpcodeDecompileError::ByteIteratorError)?;

        // Match the byte.
        match byte {
            // 0x00..0x60; Data pushes
            0x00 => Ok(Opcode::OP_FALSE(OP_FALSE)),
            0x01..=0x4b => {
                // Data length is the byte itself.
                let data_length = byte as usize;

                // Collect from bytestream data_length number of bytes.
                let data = bytecode_stream
                    .by_ref()
                    .take(data_length)
                    .collect::<Vec<u8>>();

                // Check if the data length is valid.
                if data.len() != data_length {
                    return Err(OpcodeDecompileError::InvalidPushDataLength);
                }

                // Check if the data is a minimal push.
                if data_length == 1 && !OP_PUSHDATA::check_minimal_push(&data) {
                    return Err(OpcodeDecompileError::NonMinimalDataPushError);
                }

                // Return the opcode.
                Ok(Opcode::OP_PUSHDATA(OP_PUSHDATA(data)))
            }
            0x4c => {
                // Data length is the next byte.
                let data_length = bytecode_stream
                    .next()
                    .ok_or(OpcodeDecompileError::ByteIteratorError)?
                    as usize;

                // Check if the data push is in tier 1.
                if data_length <= 75 {
                    // Should have been tier 0 (0x01..0x4b).
                    return Err(OpcodeDecompileError::InvalidDataPushTier);
                }

                // Collect from bytestream data_length number of bytes.
                let data = bytecode_stream
                    .by_ref()
                    .take(data_length)
                    .collect::<Vec<u8>>();

                // Check if the data length is valid.
                if data.len() != data_length {
                    return Err(OpcodeDecompileError::InvalidPushDataLength);
                }

                // Return the opcode.
                Ok(Opcode::OP_PUSHDATA(OP_PUSHDATA(data)))
            }
            0x4d => {
                // Collect two bytes for the data length in little endian order.
                let mut data_length = bytecode_stream
                    .next()
                    .ok_or(OpcodeDecompileError::ByteIteratorError)?
                    as u16;
                data_length |= (bytecode_stream
                    .next()
                    .ok_or(OpcodeDecompileError::ByteIteratorError)?
                    as u16)
                    << 8;

                // Check if the data push is in tier 2.
                if data_length <= 255 {
                    // Should have been tier 1 (0x4c).
                    return Err(OpcodeDecompileError::InvalidDataPushTier);
                }

                // Collect from bytestream data_length number of bytes.
                let data = bytecode_stream
                    .by_ref()
                    .take(data_length as usize)
                    .collect::<Vec<u8>>();

                // Check if the data length is valid.
                if data.len() != data_length as usize {
                    return Err(OpcodeDecompileError::InvalidPushDataLength);
                }

                // Return the opcode.
                Ok(Opcode::OP_PUSHDATA(OP_PUSHDATA(data)))
            }
            0x51 => Ok(Opcode::OP_TRUE(OP_TRUE)),
            0x52 => Ok(Opcode::OP_2(OP_2)),
            0x53 => Ok(Opcode::OP_3(OP_3)),
            0x54 => Ok(Opcode::OP_4(OP_4)),
            0x55 => Ok(Opcode::OP_5(OP_5)),
            0x56 => Ok(Opcode::OP_6(OP_6)),
            0x57 => Ok(Opcode::OP_7(OP_7)),
            0x58 => Ok(Opcode::OP_8(OP_8)),
            0x59 => Ok(Opcode::OP_9(OP_9)),
            0x5a => Ok(Opcode::OP_10(OP_10)),
            0x5b => Ok(Opcode::OP_11(OP_11)),
            0x5c => Ok(Opcode::OP_12(OP_12)),
            0x5d => Ok(Opcode::OP_13(OP_13)),
            0x5e => Ok(Opcode::OP_14(OP_14)),
            0x5f => Ok(Opcode::OP_15(OP_15)),
            0x60 => Ok(Opcode::OP_16(OP_16)),

            // Flow control
            0x61 => Ok(Opcode::OP_NOP(OP_NOP)),
            0x62 => Ok(Opcode::OP_JUMP(OP_JUMP)),
            0x63 => Ok(Opcode::OP_IF(OP_IF)),
            0x64 => Ok(Opcode::OP_NOTIF(OP_NOTIF)),
            0x65 => Ok(Opcode::OP_RETURNALL(OP_RETURNALL)),
            0x66 => Ok(Opcode::OP_RETURNSOME(OP_RETURNSOME)),
            0x67 => Ok(Opcode::OP_ELSE(OP_ELSE)),
            0x68 => Ok(Opcode::OP_ENDIF(OP_ENDIF)),
            0x69 => Ok(Opcode::OP_VERIFY(OP_VERIFY)),
            0x6a => Ok(Opcode::OP_FAIL(OP_FAIL)),

            // Altstack
            0x6b => Ok(Opcode::OP_TOALTSTACK(OP_TOALTSTACK)),
            0x6c => Ok(Opcode::OP_FROMALTSTACK(OP_FROMALTSTACK)),

            // Stack
            0x6d => Ok(Opcode::OP_2DROP(OP_2DROP)),
            0x6e => Ok(Opcode::OP_2DUP(OP_2DUP)),
            0x6f => Ok(Opcode::OP_3DUP(OP_3DUP)),
            0x70 => Ok(Opcode::OP_2OVER(OP_2OVER)),
            0x71 => Ok(Opcode::OP_2ROT(OP_2ROT)),
            0x72 => Ok(Opcode::OP_2SWAP(OP_2SWAP)),
            0x73 => Ok(Opcode::OP_IFDUP(OP_IFDUP)),
            0x74 => Ok(Opcode::OP_DEPTH(OP_DEPTH)),
            0x75 => Ok(Opcode::OP_DROP(OP_DROP)),
            0x76 => Ok(Opcode::OP_DUP(OP_DUP)),
            0x77 => Ok(Opcode::OP_NIP(OP_NIP)),
            0x78 => Ok(Opcode::OP_OVER(OP_OVER)),
            0x79 => Ok(Opcode::OP_PICK(OP_PICK)),
            0x7a => Ok(Opcode::OP_ROLL(OP_ROLL)),
            0x7b => Ok(Opcode::OP_ROT(OP_ROT)),
            0x7c => Ok(Opcode::OP_SWAP(OP_SWAP)),
            0x7d => Ok(Opcode::OP_TUCK(OP_TUCK)),

            // Splice
            0x7e => Ok(Opcode::OP_CAT(OP_CAT)),
            0x7f => Ok(Opcode::OP_SPLIT(OP_SPLIT)),
            0x80 => Ok(Opcode::OP_LEFT(OP_LEFT)),
            0x81 => Ok(Opcode::OP_RIGHT(OP_RIGHT)),
            0x82 => Ok(Opcode::OP_SIZE(OP_SIZE)),

            // Bitwise
            0x83 => Ok(Opcode::OP_INVERT(OP_INVERT)),
            0x84 => Ok(Opcode::OP_AND(OP_AND)),
            0x85 => Ok(Opcode::OP_OR(OP_OR)),
            0x86 => Ok(Opcode::OP_XOR(OP_XOR)),
            0x87 => Ok(Opcode::OP_EQUAL(OP_EQUAL)),
            0x88 => Ok(Opcode::OP_EQUALVERIFY(OP_EQUALVERIFY)),
            0x89 => Ok(Opcode::OP_REVERSE(OP_REVERSE)),

            // Arithmetic
            0x8b => Ok(Opcode::OP_1ADD(OP_1ADD)),
            0x8c => Ok(Opcode::OP_1SUB(OP_1SUB)),
            0x8d => Ok(Opcode::OP_2MUL(OP_2MUL)),
            0x8e => Ok(Opcode::OP_2DIV(OP_2DIV)),
            0x8f => Ok(Opcode::OP_ADDMOD(OP_ADDMOD)),
            0x90 => Ok(Opcode::OP_MULMOD(OP_MULMOD)),
            0x91 => Ok(Opcode::OP_NOT(OP_NOT)),
            0x92 => Ok(Opcode::OP_0NOTEQUAL(OP_0NOTEQUAL)),
            0x93 => Ok(Opcode::OP_ADD(OP_ADD)),
            0x94 => Ok(Opcode::OP_SUB(OP_SUB)),
            0x95 => Ok(Opcode::OP_MUL(OP_MUL)),
            0x96 => Ok(Opcode::OP_DIV(OP_DIV)),
            0x98 => Ok(Opcode::OP_LSHIFT(OP_LSHIFT)),
            0x99 => Ok(Opcode::OP_RSHIFT(OP_RSHIFT)),
            0x9a => Ok(Opcode::OP_BOOLAND(OP_BOOLAND)),
            0x9b => Ok(Opcode::OP_BOOLOR(OP_BOOLOR)),
            0x9c => Ok(Opcode::OP_NUMEQUAL(OP_NUMEQUAL)),
            0x9d => Ok(Opcode::OP_NUMEQUALVERIFY(OP_NUMEQUALVERIFY)),
            0x9e => Ok(Opcode::OP_NUMNOTEQUAL(OP_NUMNOTEQUAL)),
            0x9f => Ok(Opcode::OP_LESSTHAN(OP_LESSTHAN)),
            0xa0 => Ok(Opcode::OP_GREATERTHAN(OP_GREATERTHAN)),
            0xa1 => Ok(Opcode::OP_LESSTHANOREQUAL(OP_LESSTHANOREQUAL)),
            0xa2 => Ok(Opcode::OP_GREATERTHANOREQUAL(OP_GREATERTHANOREQUAL)),
            0xa3 => Ok(Opcode::OP_MIN(OP_MIN)),
            0xa4 => Ok(Opcode::OP_MAX(OP_MAX)),
            0xa5 => Ok(Opcode::OP_WITHIN(OP_WITHIN)),

            // Hashing
            0xa6 => Ok(Opcode::OP_RIPEMD160(OP_RIPEMD160)),
            0xa7 => Ok(Opcode::OP_SHA1(OP_SHA1)),
            0xa8 => Ok(Opcode::OP_SHA256(OP_SHA256)),
            0xa9 => Ok(Opcode::OP_HASH160(OP_HASH160)),
            0xaa => Ok(Opcode::OP_HASH256(OP_HASH256)),
            0xab => Ok(Opcode::OP_TAGGEDHASH(OP_TAGGEDHASH)),
            0xac => Ok(Opcode::OP_BLAKE2BVAR(OP_BLAKE2BVAR)),
            0xad => Ok(Opcode::OP_BLAKE2SVAR(OP_BLAKE2SVAR)),

            // Secp
            0xae => Ok(Opcode::OP_SECPSCALARADD(OP_SECPSCALARADD)),
            0xaf => Ok(Opcode::OP_SECPSCALARMUL(OP_SECPSCALARMUL)),
            0xb0 => Ok(Opcode::OP_SECPPOINTADD(OP_SECPPOINTADD)),
            0xb1 => Ok(Opcode::OP_SECPPOINTMUL(OP_SECPPOINTMUL)),
            0xb2 => Ok(Opcode::OP_PUSHSECPGENERATORPOINT(OP_PUSHSECPGENERATORPOINT)),
            0xb3 => Ok(Opcode::OP_ISZEROSECPSCALAR(OP_ISZEROSECPSCALAR)),
            0xb4 => Ok(Opcode::OP_ISINFINITESECPPOINT(OP_ISINFINITESECPPOINT)),

            // Digital signatures
            0xb5 => Ok(Opcode::OP_CHECKSCHNORRSIG(OP_CHECKSCHNORRSIG)),
            0xb6 => Ok(Opcode::OP_CHECKSCHNORRSIGBIP340(OP_CHECKSCHNORRSIGBIP340)),
            0xb7 => Ok(Opcode::OP_CHECKBLSSIG(OP_CHECKBLSSIG)),
            0xb8 => Ok(Opcode::OP_CHECKBLSSIGAGG(OP_CHECKBLSSIGAGG)),

            // Call info
            0xb9 => Ok(Opcode::OP_CALLER(OP_CALLER)),
            0xba => Ok(Opcode::OP_OPSBUDGET(OP_OPSBUDGET)),
            0xbb => Ok(Opcode::OP_OPSCOUNTER(OP_OPSCOUNTER)),
            0xbc => Ok(Opcode::OP_OPSPRICE(OP_OPSPRICE)),
            0xbd => Ok(Opcode::OP_TIMESTAMP(OP_TIMESTAMP)),

            // Call
            0xbe => Ok(Opcode::OP_CALL(OP_CALL)),
            0xbf => Ok(Opcode::OP_CALLEXT(OP_CALLEXT)),

            // Shadowing
            0xc0 => Ok(Opcode::OP_SHADOW_HAS_ALLOC(OP_SHADOW_HAS_ALLOC)),
            0xc1 => Ok(Opcode::OP_SHADOW_DEALLOC(OP_SHADOW_DEALLOC)),
            0xc2 => Ok(Opcode::OP_SHADOW_ALLOC_VAL(OP_SHADOW_ALLOC_VAL)),
            0xc3 => Ok(Opcode::OP_SHADOW_ALLOC(OP_SHADOW_ALLOC)),
            0xc4 => Ok(Opcode::OP_SHADOW_ALLOC_UP(OP_SHADOW_ALLOC_UP)),
            0xc5 => Ok(Opcode::OP_SHADOW_ALLOC_DOWN(OP_SHADOW_ALLOC_DOWN)),
            0xc6 => Ok(Opcode::OP_SHADOW_ALLOC_UP_ALL(OP_SHADOW_ALLOC_UP_ALL)),
            0xc7 => Ok(Opcode::OP_SHADOW_ALLOC_DOWN_ALL(OP_SHADOW_ALLOC_DOWN_ALL)),
            0xc8 => Ok(Opcode::OP_SHADOW_NUM_ALLOCS(OP_SHADOW_NUM_ALLOCS)),
            0xc9 => Ok(Opcode::OP_SHADOW_ALLOCS_SUM(OP_SHADOW_ALLOCS_SUM)),

            // Coin
            0xca => Ok(Opcode::OP_EXT_BALANCE(OP_EXT_BALANCE)),
            0xcb => Ok(Opcode::OP_SELF_BALANCE(OP_SELF_BALANCE)),
            0xcc => Ok(Opcode::OP_TRANSFER(OP_TRANSFER)),

            // Storage
            0xcd => Ok(Opcode::OP_SWRITE(OP_SWRITE)),
            0xce => Ok(Opcode::OP_SREAD(OP_SREAD)),

            // Memory
            0xd0 => Ok(Opcode::OP_MWRITE(OP_MWRITE)),
            0xd1 => Ok(Opcode::OP_MREAD(OP_MREAD)),
            0xd2 => Ok(Opcode::OP_MFREE(OP_MFREE)),

            // Undefined
            _ => Err(OpcodeDecompileError::UndefinedOpcodeError),
        }
    }
}
