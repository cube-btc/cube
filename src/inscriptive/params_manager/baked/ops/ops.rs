/// Data push operation weights.
/// ------------------------------------------------------------
// False push operation weight.
pub const OP_FALSE_OPS: u64 = 1;

// Varying-size data push operation base weight.
pub const OP_PUSHDATA_BASE_OPS: u64 = 1;

// Varying-size data push operation multiplier weight.
pub const OP_PUSHDATA_MULTIPLIER_OPS: u64 = 1;

// True push operation weight.
pub const OP_TRUE_OPS: u64 = 1;

// 2 push operation weight.
pub const OP_2_OPS: u64 = 1;

// 3 push operation weight.
pub const OP_3_OPS: u64 = 1;

// 4 push operation weight.
pub const OP_4_OPS: u64 = 1;

// 5 push operation weight.
pub const OP_5_OPS: u64 = 1;

// 6 push operation weight.
pub const OP_6_OPS: u64 = 1;

// 7 push operation weight.
pub const OP_7_OPS: u64 = 1;

// 8 push operation weight.
pub const OP_8_OPS: u64 = 1;

// 9 push operation weight.
pub const OP_9_OPS: u64 = 1;

// 10 push operation weight.
pub const OP_10_OPS: u64 = 1;

// 11 push operation weight.
pub const OP_11_OPS: u64 = 1;

// 12 push operation weight.
pub const OP_12_OPS: u64 = 1;

// 13 push operation weight.
pub const OP_13_OPS: u64 = 1;

// 14 push operation weight.
pub const OP_14_OPS: u64 = 1;

// 15 push operation weight.
pub const OP_15_OPS: u64 = 1;

// 16 push operation weight.
pub const OP_16_OPS: u64 = 1;

/// Flow control operation weights.
/// ------------------------------------------------------------
// NOP operation weight.
pub const OP_NOP_OPS: u64 = 1;

// Jump operation weight.
pub const OP_JUMP_OPS: u64 = 1;

// If operation weight.
pub const OP_IF_OPS: u64 = 1;

// Not if operation weight.
pub const OP_NOTIF_OPS: u64 = 1;

// Return all operation weight.
pub const OP_RETURNALL_OPS: u64 = 1;

// Return some operation weight.
pub const OP_RETURNSOME_OPS: u64 = 1;

// Else operation weight.
pub const OP_ELSE_OPS: u64 = 1;

// End if operation weight.
pub const OP_ENDIF_OPS: u64 = 1;

// Verify operation weight.
pub const OP_VERIFY_OPS: u64 = 1;

// Fail operation weight.
pub const OP_FAIL_OPS: u64 = 1;

/// Altstack operation weights.
/// ------------------------------------------------------------
// To altstack operation weight.
pub const OP_TOALTSTACK_OPS: u64 = 1;

// From altstack operation weight.
pub const OP_FROMALTSTACK_OPS: u64 = 1;

/// Stack operation weights.
/// ------------------------------------------------------------
// 2 drop operation weight.
pub const OP_2DROP_OPS: u64 = 1;

// 2 dup operation weight.
pub const OP_2DUP_OPS: u64 = 1;

// 3 dup operation weight.
pub const OP_3DUP_OPS: u64 = 1;

// 2 over operation weight.
pub const OP_2OVER_OPS: u64 = 1;

// 2 rot operation weight.
pub const OP_2ROT_OPS: u64 = 1;

// 2 swap operation weight.
pub const OP_2SWAP_OPS: u64 = 1;

// If dup operation weight.
pub const OP_IFDUP_OPS: u64 = 1;

// Depth operation weight.
pub const OP_DEPTH_OPS: u64 = 1;

// Drop operation weight.
pub const OP_DROP_OPS: u64 = 1;

// Dup operation weight.
pub const OP_DUP_OPS: u64 = 1;

// Nip operation weight.
pub const OP_NIP_OPS: u64 = 1;

// Over operation weight.
pub const OP_OVER_OPS: u64 = 1;

// Pick operation weight.
pub const OP_PICK_OPS: u64 = 1;

// Roll operation weight.
pub const OP_ROLL_OPS: u64 = 1;

// Rot operation weight.
pub const OP_ROT_OPS: u64 = 1;

// Swap operation weight.
pub const OP_SWAP_OPS: u64 = 1;

// Tuck operation weight.
pub const OP_TUCK_OPS: u64 = 1;

/// Splice operation weights.
/// ------------------------------------------------------------
// Cat operation weight.
pub const OP_CAT_OPS: u64 = 2;

// Split operation weight.
pub const OP_SPLIT_OPS: u64 = 2;

// Left operation weight.
pub const OP_LEFT_OPS: u64 = 2;

// Right operation weight.
pub const OP_RIGHT_OPS: u64 = 2;

// Size operation weight.
pub const OP_SIZE_OPS: u64 = 2;

/// Bitwise operation weights.
/// ------------------------------------------------------------
// Invert operation weight.
pub const OP_INVERT_OPS: u64 = 2;

// And operation weight.
pub const OP_AND_OPS: u64 = 2;

// Or operation weight.
pub const OP_OR_OPS: u64 = 2;

// Xor operation weight.
pub const OP_XOR_OPS: u64 = 2;

// Equal operation weight.
pub const OP_EQUAL_OPS: u64 = 1;

// Equal verify operation weight.
pub const OP_EQUALVERIFY_OPS: u64 = 2;

// Reverse operation weight.
pub const OP_REVERSE_OPS: u64 = 3;

/// Arithmetic operation weights.
/// ------------------------------------------------------------
// 1 add operation weight.
pub const OP_1ADD_OPS: u64 = 3;

// 1 sub operation weight.
pub const OP_1SUB_OPS: u64 = 3;

// 2 mul operation weight.
pub const OP_2MUL_OPS: u64 = 5;

// 2 div operation weight.
pub const OP_2DIV_OPS: u64 = 5;

// Add mod operation weight.
pub const OP_ADDMOD_OPS: u64 = 3;

// Mul mod operation weight.
pub const OP_MULMOD_OPS: u64 = 3;

// Not operation weight.
pub const OP_NOT_OPS: u64 = 1;

// 0 not equal operation weight.
pub const OP_0NOTEQUAL_OPS: u64 = 1;

// Add operation weight.
pub const OP_ADD_OPS: u64 = 3;

// Sub operation weight.
pub const OP_SUB_OPS: u64 = 3;

// Mul operation weight.
pub const OP_MUL_OPS: u64 = 5;

// Div operation weight.
pub const OP_DIV_OPS: u64 = 5;

// Lshift operation weight.
pub const OP_LSHIFT_OPS: u64 = 3;

// Rshift operation weight.
pub const OP_RSHIFT_OPS: u64 = 3;

// Bool and operation weight.
pub const OP_BOOLAND_OPS: u64 = 1;

// Bool or operation weight.
pub const OP_BOOLOR_OPS: u64 = 1;

// Num equal operation weight.
pub const OP_NUMEQUAL_OPS: u64 = 1;

// Num equal verify operation weight.
pub const OP_NUMEQUALVERIFY_OPS: u64 = 2;

// Num not equal operation weight.
pub const OP_NUMNOTEQUAL_OPS: u64 = 1;

// Less than operation weight.
pub const OP_LESSTHAN_OPS: u64 = 1;

// Greater than operation weight.
pub const OP_GREATERTHAN_OPS: u64 = 1;

// Less than or equal operation weight.
pub const OP_LESSTHANOREQUAL_OPS: u64 = 1;

// Greater than or equal operation weight.
pub const OP_GREATERTHANOREQUAL_OPS: u64 = 1;

// Min operation weight.
pub const OP_MIN_OPS: u64 = 1;

// Max operation weight.
pub const OP_MAX_OPS: u64 = 1;

// Within operation weight.
pub const OP_WITHIN_OPS: u64 = 1;

/// Digest operation weights.
/// ------------------------------------------------------------
// RIPEMD160 operation weight.
pub const OP_RIPEMD160_OPS: u64 = 30;

// SHA1 operation weight.
pub const OP_SHA1_OPS: u64 = 30;

// SHA256 operation weight.
pub const OP_SHA256_OPS: u64 = 42;

// Hash160 operation weight.
pub const OP_HASH160_OPS: u64 = 72;

// Hash256 operation weight.
pub const OP_HASH256_OPS: u64 = 84;

// Tagged hash operation weight.
pub const OP_TAGGEDHASH_OPS: u64 = 42;

// Blake2b var operation base weight.
pub const OP_BLAKE2BVAR_BASE_OPS: u64 = 10;
// Blake2b var operation multiplier weight.
pub const OP_BLAKE2BVAR_MULTIPLIER_OPS: u64 = 1;

// Blake2s var operation base weight.
pub const OP_BLAKE2SVAR_BASE_OPS: u64 = 10;
// Blake2s var operation multiplier weight.
pub const OP_BLAKE2SVAR_MULTIPLIER_OPS: u64 = 1;

/// Secp operation weights.
/// ------------------------------------------------------------
// Secp scalar add operation weight.
pub const OP_SECPSCALARADD_OPS: u64 = 10;

// Secp scalar mul operation weight.
pub const OP_SECPSCALARMUL_OPS: u64 = 10;

// Secp point add operation weight.
pub const OP_SECPPOINTADD_OPS: u64 = 50;

// Secp point mul operation weight.
pub const OP_SECPPOINTMUL_OPS: u64 = 50;

// Push secp generator point operation weight.
pub const OP_PUSHSECPGENERATORPOINT_OPS: u64 = 50;

// Is zero secp scalar operation weight.
pub const OP_ISZEROSECPSCALAR_OPS: u64 = 50;

// Is infinite secp point operation weight.
pub const OP_ISINFINITESECPPOINT_OPS: u64 = 50;

/// Digital signature operation weights.
/// ------------------------------------------------------------
// Check schnorr sig operation weight.
pub const OP_CHECKSCHNORRSIG_OPS: u64 = 100;

// Check schnorr sig BIP340 operation weight.
pub const OP_CHECKSCHNORRSIGBIP340_OPS: u64 = 100;

// Check BLS sig operation weight.
pub const OP_CHECKBLSSIG_OPS: u64 = 100;

// Check BLS sig agg operation base weight.
pub const OP_CHECKBLSSIGAGG_BASE_OPS: u64 = 100;
// Check BLS sig agg operation multiplier weight.
pub const OP_CHECKBLSSIGAGG_MULTIPLIER_OPS: u64 = 50;

/// Call info operation weights.
/// ------------------------------------------------------------
// Caller operation weight.
pub const OP_CALLER_OPS: u64 = 1;

// Ops budget operation weight.
pub const OP_OPSBUDGET_OPS: u64 = 1;

// Ops counter operation weight.
pub const OP_OPSCOUNTER_OPS: u64 = 1;

// Ops price operation weight.
pub const OP_OPSPRICE_OPS: u64 = 1;

// Timestamp operation weight.
pub const OP_TIMESTAMP_OPS: u64 = 1;

/// Call operation weights.
/// ------------------------------------------------------------
// Call operation weight.
pub const OP_CALL_OPS: u64 = 5;

// Call ext operation weight.
pub const OP_CALLEXT_OPS: u64 = 50;

/// Shadowing operation weights.
/// ------------------------------------------------------------
// Shadow alloc operation weight.
pub const OP_SHADOW_ALLOC_OPS: u64 = 1000;

// Shadow dealloc operation weight.
pub const OP_SHADOW_DEALLOC_OPS: u64 = 900;

// Shadow has alloc operation weight.
pub const OP_SHADOW_HAS_ALLOC_OPS: u64 = 1;

// Shadow alloc val operation weight.
pub const OP_SHADOW_ALLOC_VAL_OPS: u64 = 1;

// Shadow up operation weight.
pub const OP_SHADOW_UP_OPS: u64 = 5;

// Shadow down operation weight.
pub const OP_SHADOW_DOWN_OPS: u64 = 5;

// Shadow up all operation weight.
pub const OP_SHADOW_UP_ALL_OPS: u64 = 50;

// Shadow down all operation weight.
pub const OP_SHADOW_DOWN_ALL_OPS: u64 = 50;

// Shadow num allocs operation weight.
pub const OP_SHADOW_NUM_ALLOCS_OPS: u64 = 1;

// Shadow allocs sum operation weight.
pub const OP_SHADOW_ALLOCS_SUM_OPS: u64 = 1;

/// Coin operation weights.
/// ------------------------------------------------------------
// Ext balance operation weight.
pub const OP_EXT_BALANCE_OPS: u64 = 1;

// Self balance operation weight.
pub const OP_SELF_BALANCE_OPS: u64 = 1;

// Transfer operation weight.
pub const OP_TRANSFER_OPS: u64 = 10;

/// Storage operation weights.
/// ------------------------------------------------------------
// Storage write operation weight.
pub const OP_SWRITE_OPS: u64 = 50;

// Storage read operation weight.
pub const OP_SREAD_OPS: u64 = 50;

/// Memory operation weights.
/// ------------------------------------------------------------
// Memory write operation weight.
pub const OP_MWRITE_OPS: u64 = 5;

// Memory read operation weight.
pub const OP_MREAD_OPS: u64 = 5;

// Memory free operation weight.
pub const OP_MFREE_OPS: u64 = 1;
