/// Baked, initial opcode execution fees parameters.
/// Key prefix: 0x03
/// ------------------------------------------------------------

/// Data push operation weights.
/// ---------------------------------------------
// False push operation weight key & value.
pub const OP_FALSE_OPS_KEY: [u8; 2] = [0x03, 0x00];
pub const OP_FALSE_OPS_VALUE: u64 = 1;

// Varying-size data push operation base weight key & value.
pub const OP_PUSHDATA_BASE_OPS_KEY: [u8; 2] = [0x03, 0x01];
pub const OP_PUSHDATA_BASE_OPS_VALUE: u64 = 1;

// Varying-size data push operation multiplier weight key & value.
pub const OP_PUSHDATA_MULTIPLIER_OPS_KEY: [u8; 2] = [0x03, 0x02];
pub const OP_PUSHDATA_MULTIPLIER_OPS_VALUE: u64 = 1;

// True push operation weight key & value.
pub const OP_TRUE_OPS_KEY: [u8; 2] = [0x03, 0x03];
pub const OP_TRUE_OPS_VALUE: u64 = 1;

// 2 push operation weight key & value.
pub const OP_2_OPS_KEY: [u8; 2] = [0x03, 0x04];
pub const OP_2_OPS_VALUE: u64 = 1;

// 3 push operation weight key & value.
pub const OP_3_OPS_KEY: [u8; 2] = [0x03, 0x05];
pub const OP_3_OPS_VALUE: u64 = 1;

// 4 push operation weight key & value.
pub const OP_4_OPS_KEY: [u8; 2] = [0x03, 0x06];
pub const OP_4_OPS_VALUE: u64 = 1;

// 5 push operation weight key & value.
pub const OP_5_OPS_KEY: [u8; 2] = [0x03, 0x07];
pub const OP_5_OPS_VALUE: u64 = 1;

// 6 push operation weight key & value.
pub const OP_6_OPS_KEY: [u8; 2] = [0x03, 0x08];
pub const OP_6_OPS_VALUE: u64 = 1;

// 7 push operation weight key & value.
pub const OP_7_OPS_KEY: [u8; 2] = [0x03, 0x09];
pub const OP_7_OPS_VALUE: u64 = 1;

// 8 push operation weight key & value.
pub const OP_8_OPS_KEY: [u8; 2] = [0x03, 0x0A];
pub const OP_8_OPS_VALUE: u64 = 1;

// 9 push operation weight key & value.
pub const OP_9_OPS_KEY: [u8; 2] = [0x03, 0x0B];
pub const OP_9_OPS_VALUE: u64 = 1;

// 10 push operation weight key & value.
pub const OP_10_OPS_KEY: [u8; 2] = [0x03, 0x0C];
pub const OP_10_OPS_VALUE: u64 = 1;

// 11 push operation weight key & value.
pub const OP_11_OPS_KEY: [u8; 2] = [0x03, 0x0D];
pub const OP_11_OPS_VALUE: u64 = 1;

// 12 push operation weight key & value.
pub const OP_12_OPS_KEY: [u8; 2] = [0x03, 0x0E];
pub const OP_12_OPS_VALUE: u64 = 1;

// 13 push operation weight key & value.
pub const OP_13_OPS_KEY: [u8; 2] = [0x03, 0x0F];
pub const OP_13_OPS_VALUE: u64 = 1;

// 14 push operation weight key & value.
pub const OP_14_OPS_KEY: [u8; 2] = [0x03, 0x10];
pub const OP_14_OPS_VALUE: u64 = 1;

// 15 push operation weight key & value.
pub const OP_15_OPS_KEY: [u8; 2] = [0x03, 0x11];
pub const OP_15_OPS_VALUE: u64 = 1;

// 16 push operation weight key & value.
pub const OP_16_OPS_KEY: [u8; 2] = [0x03, 0x12];
pub const OP_16_OPS_VALUE: u64 = 1;

/// Flow control operation weights.
/// ---------------------------------------------
// NOP operation weight key & value.
pub const OP_NOP_OPS_KEY: [u8; 2] = [0x03, 0x13];
pub const OP_NOP_OPS_VALUE: u64 = 1;

// Jump operation weight key & value.
pub const OP_JUMP_OPS_KEY: [u8; 2] = [0x03, 0x14];
pub const OP_JUMP_OPS_VALUE: u64 = 1;

// If operation weight key & value.
pub const OP_IF_OPS_KEY: [u8; 2] = [0x03, 0x15];
pub const OP_IF_OPS_VALUE: u64 = 1;

// Not if operation weight key & value.
pub const OP_NOTIF_OPS_KEY: [u8; 2] = [0x03, 0x16];
pub const OP_NOTIF_OPS_VALUE: u64 = 1;

// Return all operation weight key & value.
pub const OP_RETURNALL_OPS_KEY: [u8; 2] = [0x03, 0x17];
pub const OP_RETURNALL_OPS_VALUE: u64 = 1;

// Return some operation weight key & value.
pub const OP_RETURNSOME_OPS_KEY: [u8; 2] = [0x03, 0x18];
pub const OP_RETURNSOME_OPS_VALUE: u64 = 1;

// Else operation weight key & value.
pub const OP_ELSE_OPS_KEY: [u8; 2] = [0x03, 0x19];
pub const OP_ELSE_OPS_VALUE: u64 = 1;

// End if operation weight key & value.
pub const OP_ENDIF_OPS_KEY: [u8; 2] = [0x03, 0x1A];
pub const OP_ENDIF_OPS_VALUE: u64 = 1;

// Verify operation weight key & value.
pub const OP_VERIFY_OPS_KEY: [u8; 2] = [0x03, 0x1B];
pub const OP_VERIFY_OPS_VALUE: u64 = 1;

// Fail operation weight key & value.
pub const OP_FAIL_OPS_KEY: [u8; 2] = [0x03, 0x1C];
pub const OP_FAIL_OPS_VALUE: u64 = 1;

/// Altstack operation weights.
/// ---------------------------------------------
// To altstack operation weight key & value.
pub const OP_TOALTSTACK_OPS_KEY: [u8; 2] = [0x03, 0x1D];
pub const OP_TOALTSTACK_OPS_VALUE: u64 = 1;

// From altstack operation weight key & value.
pub const OP_FROMALTSTACK_OPS_KEY: [u8; 2] = [0x03, 0x1E];
pub const OP_FROMALTSTACK_OPS_VALUE: u64 = 1;

/// Stack operation weights.
/// ---------------------------------------------
// 2 drop operation weight key & value.
pub const OP_2DROP_OPS_KEY: [u8; 2] = [0x03, 0x1F];
pub const OP_2DROP_OPS_VALUE: u64 = 1;

// 2 dup operation weight key & value.
pub const OP_2DUP_OPS_KEY: [u8; 2] = [0x03, 0x20];
pub const OP_2DUP_OPS_VALUE: u64 = 1;

// 3 dup operation weight key & value.
pub const OP_3DUP_OPS_KEY: [u8; 2] = [0x03, 0x21];
pub const OP_3DUP_OPS_VALUE: u64 = 1;

// 2 over operation weight key & value.
pub const OP_2OVER_OPS_KEY: [u8; 2] = [0x03, 0x22];
pub const OP_2OVER_OPS_VALUE: u64 = 1;

// 2 rot operation weight key & value.
pub const OP_2ROT_OPS_KEY: [u8; 2] = [0x03, 0x23];
pub const OP_2ROT_OPS_VALUE: u64 = 1;

// 2 swap operation weight key & value.
pub const OP_2SWAP_OPS_KEY: [u8; 2] = [0x03, 0x24];
pub const OP_2SWAP_OPS_VALUE: u64 = 1;

// If dup operation weight key & value.
pub const OP_IFDUP_OPS_KEY: [u8; 2] = [0x03, 0x25];
pub const OP_IFDUP_OPS_VALUE: u64 = 1;

// Depth operation weight key & value.
pub const OP_DEPTH_OPS_KEY: [u8; 2] = [0x03, 0x26];
pub const OP_DEPTH_OPS_VALUE: u64 = 1;

// Drop operation weight key & value.
pub const OP_DROP_OPS_KEY: [u8; 2] = [0x03, 0x27];
pub const OP_DROP_OPS_VALUE: u64 = 1;

// Dup operation weight key & value.
pub const OP_DUP_OPS_KEY: [u8; 2] = [0x03, 0x28];
pub const OP_DUP_OPS_VALUE: u64 = 1;

// Nip operation weight key & value.
pub const OP_NIP_OPS_KEY: [u8; 2] = [0x03, 0x29];
pub const OP_NIP_OPS_VALUE: u64 = 1;

// Over operation weight key & value.
pub const OP_OVER_OPS_KEY: [u8; 2] = [0x03, 0x2A];
pub const OP_OVER_OPS_VALUE: u64 = 1;

// Pick operation weight key & value.
pub const OP_PICK_OPS_KEY: [u8; 2] = [0x03, 0x2B];
pub const OP_PICK_OPS_VALUE: u64 = 1;

// Roll operation weight key & value.
pub const OP_ROLL_OPS_KEY: [u8; 2] = [0x03, 0x2C];
pub const OP_ROLL_OPS_VALUE: u64 = 1;

// Rot operation weight key & value.
pub const OP_ROT_OPS_KEY: [u8; 2] = [0x03, 0x2D];
pub const OP_ROT_OPS_VALUE: u64 = 1;

// Swap operation weight key & value.
pub const OP_SWAP_OPS_KEY: [u8; 2] = [0x03, 0x2E];
pub const OP_SWAP_OPS_VALUE: u64 = 1;

// Tuck operation weight key & value.
pub const OP_TUCK_OPS_KEY: [u8; 2] = [0x03, 0x2F];
pub const OP_TUCK_OPS_VALUE: u64 = 1;

/// Splice operation weights.
/// ---------------------------------------------
// Cat operation weight key & value.
pub const OP_CAT_OPS_KEY: [u8; 2] = [0x03, 0x30];
pub const OP_CAT_OPS_VALUE: u64 = 2;

// Split operation weight key & value.
pub const OP_SPLIT_OPS_KEY: [u8; 2] = [0x03, 0x31];
pub const OP_SPLIT_OPS_VALUE: u64 = 2;

// Left operation weight key & value.
pub const OP_LEFT_OPS_KEY: [u8; 2] = [0x03, 0x32];
pub const OP_LEFT_OPS_VALUE: u64 = 2;

// Right operation weight key & value.
pub const OP_RIGHT_OPS_KEY: [u8; 2] = [0x03, 0x33];
pub const OP_RIGHT_OPS_VALUE: u64 = 2;

// Size operation weight key & value.
pub const OP_SIZE_OPS_KEY: [u8; 2] = [0x03, 0x34];
pub const OP_SIZE_OPS_VALUE: u64 = 2;

/// Bitwise operation weights.
/// ---------------------------------------------
// Invert operation weight key & value.
pub const OP_INVERT_OPS_KEY: [u8; 2] = [0x03, 0x35];
pub const OP_INVERT_OPS_VALUE: u64 = 2;

// And operation weight key & value.
pub const OP_AND_OPS_KEY: [u8; 2] = [0x03, 0x36];
pub const OP_AND_OPS_VALUE: u64 = 2;

// Or operation weight key & value.
pub const OP_OR_OPS_KEY: [u8; 2] = [0x03, 0x37];
pub const OP_OR_OPS_VALUE: u64 = 2;

// Xor operation weight key & value.
pub const OP_XOR_OPS_KEY: [u8; 2] = [0x03, 0x38];
pub const OP_XOR_OPS_VALUE: u64 = 2;

// Equal operation weight key & value.
pub const OP_EQUAL_OPS_KEY: [u8; 2] = [0x03, 0x39];
pub const OP_EQUAL_OPS_VALUE: u64 = 1;

// Equal verify operation weight key & value.
pub const OP_EQUALVERIFY_OPS_KEY: [u8; 2] = [0x03, 0x3A];
pub const OP_EQUALVERIFY_OPS_VALUE: u64 = 2;

// Reverse operation weight key & value.
pub const OP_REVERSE_OPS_KEY: [u8; 2] = [0x03, 0x3B];
pub const OP_REVERSE_OPS_VALUE: u64 = 3;

/// Arithmetic operation weights.
/// ---------------------------------------------
// 1 add operation weight key & value.
pub const OP_1ADD_OPS_KEY: [u8; 2] = [0x03, 0x3C];
pub const OP_1ADD_OPS_VALUE: u64 = 3;

// 1 sub operation weight key & value.
pub const OP_1SUB_OPS_KEY: [u8; 2] = [0x03, 0x3D];
pub const OP_1SUB_OPS_VALUE: u64 = 3;

// 2 mul operation weight key & value.
pub const OP_2MUL_OPS_KEY: [u8; 2] = [0x03, 0x3E];
pub const OP_2MUL_OPS_VALUE: u64 = 5;

// 2 div operation weight key & value.
pub const OP_2DIV_OPS_KEY: [u8; 2] = [0x03, 0x3F];
pub const OP_2DIV_OPS_VALUE: u64 = 5;

// Add mod operation weight key & value.
pub const OP_ADDMOD_OPS_KEY: [u8; 2] = [0x03, 0x40];
pub const OP_ADDMOD_OPS_VALUE: u64 = 3;

// Mul mod operation weight key & value.
pub const OP_MULMOD_OPS_KEY: [u8; 2] = [0x03, 0x41];
pub const OP_MULMOD_OPS_VALUE: u64 = 3;

// Not operation weight key & value.
pub const OP_NOT_OPS_KEY: [u8; 2] = [0x03, 0x42];
pub const OP_NOT_OPS_VALUE: u64 = 1;

// 0 not equal operation weight key & value.
pub const OP_0NOTEQUAL_OPS_KEY: [u8; 2] = [0x03, 0x43];
pub const OP_0NOTEQUAL_OPS_VALUE: u64 = 1;

// Add operation weight key & value.
pub const OP_ADD_OPS_KEY: [u8; 2] = [0x03, 0x44];
pub const OP_ADD_OPS_VALUE: u64 = 3;

// Sub operation weight key & value.
pub const OP_SUB_OPS_KEY: [u8; 2] = [0x03, 0x45];
pub const OP_SUB_OPS_VALUE: u64 = 3;

// Mul operation weight key & value.
pub const OP_MUL_OPS_KEY: [u8; 2] = [0x03, 0x46];
pub const OP_MUL_OPS_VALUE: u64 = 5;

// Div operation weight key & value.
pub const OP_DIV_OPS_KEY: [u8; 2] = [0x03, 0x47];
pub const OP_DIV_OPS_VALUE: u64 = 5;

// Lshift operation weight key & value.
pub const OP_LSHIFT_OPS_KEY: [u8; 2] = [0x03, 0x48];
pub const OP_LSHIFT_OPS_VALUE: u64 = 3;

// Rshift operation weight key & value.
pub const OP_RSHIFT_OPS_KEY: [u8; 2] = [0x03, 0x49];
pub const OP_RSHIFT_OPS_VALUE: u64 = 3;

// Bool and operation weight key & value.
pub const OP_BOOLAND_OPS_KEY: [u8; 2] = [0x03, 0x4A];
pub const OP_BOOLAND_OPS_VALUE: u64 = 1;

// Bool or operation weight key & value.
pub const OP_BOOLOR_OPS_KEY: [u8; 2] = [0x03, 0x4B];
pub const OP_BOOLOR_OPS_VALUE: u64 = 1;

// Num equal operation weight key & value.
pub const OP_NUMEQUAL_OPS_KEY: [u8; 2] = [0x03, 0x4C];
pub const OP_NUMEQUAL_OPS_VALUE: u64 = 1;

// Num equal verify operation weight key & value.
pub const OP_NUMEQUALVERIFY_OPS_KEY: [u8; 2] = [0x03, 0x4D];
pub const OP_NUMEQUALVERIFY_OPS_VALUE: u64 = 2;

// Num not equal operation weight key & value.
pub const OP_NUMNOTEQUAL_OPS_KEY: [u8; 2] = [0x03, 0x4E];
pub const OP_NUMNOTEQUAL_OPS_VALUE: u64 = 1;

// Less than operation weight key & value.
pub const OP_LESSTHAN_OPS_KEY: [u8; 2] = [0x03, 0x4F];
pub const OP_LESSTHAN_OPS_VALUE: u64 = 1;

// Greater than operation weight key & value.
pub const OP_GREATERTHAN_OPS_KEY: [u8; 2] = [0x03, 0x50];
pub const OP_GREATERTHAN_OPS_VALUE: u64 = 1;

// Less than or equal operation weight key & value.
pub const OP_LESSTHANOREQUAL_OPS_KEY: [u8; 2] = [0x03, 0x51];
pub const OP_LESSTHANOREQUAL_OPS_VALUE: u64 = 1;

// Greater than or equal operation weight key & value.
pub const OP_GREATERTHANOREQUAL_OPS_KEY: [u8; 2] = [0x03, 0x52];
pub const OP_GREATERTHANOREQUAL_OPS_VALUE: u64 = 1;

// Min operation weight key & value.
pub const OP_MIN_OPS_KEY: [u8; 2] = [0x03, 0x53];
pub const OP_MIN_OPS_VALUE: u64 = 1;

// Max operation weight key & value.
pub const OP_MAX_OPS_KEY: [u8; 2] = [0x03, 0x54];
pub const OP_MAX_OPS_VALUE: u64 = 1;

// Within operation weight key & value.
pub const OP_WITHIN_OPS_KEY: [u8; 2] = [0x03, 0x55];
pub const OP_WITHIN_OPS_VALUE: u64 = 1;

/// Digest operation weights.
/// ---------------------------------------------
// RIPEMD160 operation weight key & value.
pub const OP_RIPEMD160_OPS_KEY: [u8; 2] = [0x03, 0x56];
pub const OP_RIPEMD160_OPS_VALUE: u64 = 30;

// SHA1 operation weight key & value.
pub const OP_SHA1_OPS_KEY: [u8; 2] = [0x03, 0x57];
pub const OP_SHA1_OPS_VALUE: u64 = 30;

// SHA256 operation weight key & value.
pub const OP_SHA256_OPS_KEY: [u8; 2] = [0x03, 0x58];
pub const OP_SHA256_OPS_VALUE: u64 = 42;

// Hash160 operation weight key & value.
pub const OP_HASH160_OPS_KEY: [u8; 2] = [0x03, 0x59];
pub const OP_HASH160_OPS_VALUE: u64 = 72;

// Hash256 operation weight key & value.
pub const OP_HASH256_OPS_KEY: [u8; 2] = [0x03, 0x5A];
pub const OP_HASH256_OPS_VALUE: u64 = 84;

// Tagged hash operation weight key & value.
pub const OP_TAGGEDHASH_OPS_KEY: [u8; 2] = [0x03, 0x5B];
pub const OP_TAGGEDHASH_OPS_VALUE: u64 = 42;

// Blake2b var operation base weight key & value.
pub const OP_BLAKE2BVAR_BASE_OPS_KEY: [u8; 2] = [0x03, 0x5C];
pub const OP_BLAKE2BVAR_BASE_OPS_VALUE: u64 = 10;

// Blake2b var operation multiplier weight key & value.
pub const OP_BLAKE2BVAR_MULTIPLIER_OPS_KEY: [u8; 2] = [0x03, 0x5D];
pub const OP_BLAKE2BVAR_MULTIPLIER_OPS_VALUE: u64 = 1;

// Blake2s var operation base weight key & value.
pub const OP_BLAKE2SVAR_BASE_OPS_KEY: [u8; 2] = [0x03, 0x5E];
pub const OP_BLAKE2SVAR_BASE_OPS_VALUE: u64 = 10;

// Blake2s var operation multiplier weight key & value.
pub const OP_BLAKE2SVAR_MULTIPLIER_OPS_KEY: [u8; 2] = [0x03, 0x5F];
pub const OP_BLAKE2SVAR_MULTIPLIER_OPS_VALUE: u64 = 1;

/// Secp operation weights.
/// ---------------------------------------------
// Secp scalar add operation weight key & value.
pub const OP_SECPSCALARADD_OPS_KEY: [u8; 2] = [0x03, 0x60];
pub const OP_SECPSCALARADD_OPS_VALUE: u64 = 10;

// Secp scalar mul operation weight key & value.
pub const OP_SECPSCALARMUL_OPS_KEY: [u8; 2] = [0x03, 0x61];
pub const OP_SECPSCALARMUL_OPS_VALUE: u64 = 10;

// Secp point add operation weight key & value.
pub const OP_SECPPOINTADD_OPS_KEY: [u8; 2] = [0x03, 0x62];
pub const OP_SECPPOINTADD_OPS_VALUE: u64 = 50;

// Secp point mul operation weight key & value.
pub const OP_SECPPOINTMUL_OPS_KEY: [u8; 2] = [0x03, 0x63];
pub const OP_SECPPOINTMUL_OPS_VALUE: u64 = 50;

// Push secp generator point operation weight key & value.
pub const OP_PUSHSECPGENERATORPOINT_OPS_KEY: [u8; 2] = [0x03, 0x64];
pub const OP_PUSHSECPGENERATORPOINT_OPS_VALUE: u64 = 50;

// Is zero secp scalar operation weight key & value.
pub const OP_ISZEROSECPSCALAR_OPS_KEY: [u8; 2] = [0x03, 0x65];
pub const OP_ISZEROSECPSCALAR_OPS_VALUE: u64 = 50;

// Is infinite secp point operation weight key & value.
pub const OP_ISINFINITESECPPOINT_OPS_KEY: [u8; 2] = [0x03, 0x66];
pub const OP_ISINFINITESECPPOINT_OPS_VALUE: u64 = 50;

/// Digital signature operation weights.
/// ---------------------------------------------
// Check schnorr sig operation weight key & value.
pub const OP_CHECKSCHNORRSIG_OPS_KEY: [u8; 2] = [0x03, 0x67];
pub const OP_CHECKSCHNORRSIG_OPS_VALUE: u64 = 100;

// Check schnorr sig BIP340 operation weight key & value.
pub const OP_CHECKSCHNORRSIGBIP340_OPS_KEY: [u8; 2] = [0x03, 0x68];
pub const OP_CHECKSCHNORRSIGBIP340_OPS_VALUE: u64 = 100;

// Check BLS sig operation weight key & value.
pub const OP_CHECKBLSSIG_OPS_KEY: [u8; 2] = [0x03, 0x69];
pub const OP_CHECKBLSSIG_OPS_VALUE: u64 = 100;

// Check BLS sig agg operation base weight key & value.
pub const OP_CHECKBLSSIGAGG_BASE_OPS_KEY: [u8; 2] = [0x03, 0x6A];
pub const OP_CHECKBLSSIGAGG_BASE_OPS_VALUE: u64 = 100;

// Check BLS sig agg operation multiplier weight key & value.
pub const OP_CHECKBLSSIGAGG_MULTIPLIER_OPS_KEY: [u8; 2] = [0x03, 0x6B];
pub const OP_CHECKBLSSIGAGG_MULTIPLIER_OPS_VALUE: u64 = 50;

/// Call info operation weights.
/// ---------------------------------------------
// Caller operation weight key & value.
pub const OP_CALLER_OPS_KEY: [u8; 2] = [0x03, 0x6C];
pub const OP_CALLER_OPS_VALUE: u64 = 1;

// Ops budget operation weight key & value.
pub const OP_OPSBUDGET_OPS_KEY: [u8; 2] = [0x03, 0x6D];
pub const OP_OPSBUDGET_OPS_VALUE: u64 = 1;

// Ops counter operation weight key & value.
pub const OP_OPSCOUNTER_OPS_KEY: [u8; 2] = [0x03, 0x6E];
pub const OP_OPSCOUNTER_OPS_VALUE: u64 = 1;

// Ops price operation weight key & value.
pub const OP_OPSPRICE_OPS_KEY: [u8; 2] = [0x03, 0x6F];
pub const OP_OPSPRICE_OPS_VALUE: u64 = 1;

// Timestamp operation weight key & value.
pub const OP_TIMESTAMP_OPS_KEY: [u8; 2] = [0x03, 0x70];
pub const OP_TIMESTAMP_OPS_VALUE: u64 = 1;

/// Call operation weights.
/// ---------------------------------------------
// Call operation weight key & value.
pub const OP_CALL_OPS_KEY: [u8; 2] = [0x03, 0x71];
pub const OP_CALL_OPS_VALUE: u64 = 5;

// Call ext operation weight key & value.
pub const OP_CALLEXT_OPS_KEY: [u8; 2] = [0x03, 0x72];
pub const OP_CALLEXT_OPS_VALUE: u64 = 50;

/// Shadowing operation weights.
/// ---------------------------------------------
// Shadow alloc operation weight key & value.
pub const OP_SHADOW_ALLOC_OPS_KEY: [u8; 2] = [0x03, 0x73];
pub const OP_SHADOW_ALLOC_OPS_VALUE: u64 = 1000;

// Shadow dealloc operation weight key & value.
pub const OP_SHADOW_DEALLOC_OPS_KEY: [u8; 2] = [0x03, 0x74];
pub const OP_SHADOW_DEALLOC_OPS_VALUE: u64 = 900;

// Shadow has alloc operation weight key & value.
pub const OP_SHADOW_HAS_ALLOC_OPS_KEY: [u8; 2] = [0x03, 0x75];
pub const OP_SHADOW_HAS_ALLOC_OPS_VALUE: u64 = 1;

// Shadow alloc val operation weight key & value.
pub const OP_SHADOW_ALLOC_VAL_OPS_KEY: [u8; 2] = [0x03, 0x76];
pub const OP_SHADOW_ALLOC_VAL_OPS_VALUE: u64 = 1;

// Shadow up operation weight key & value.
pub const OP_SHADOW_UP_OPS_KEY: [u8; 2] = [0x03, 0x77];
pub const OP_SHADOW_UP_OPS_VALUE: u64 = 5;

// Shadow down operation weight key & value.
pub const OP_SHADOW_DOWN_OPS_KEY: [u8; 2] = [0x03, 0x78];
pub const OP_SHADOW_DOWN_OPS_VALUE: u64 = 5;

// Shadow up all operation weight key & value.
pub const OP_SHADOW_UP_ALL_OPS_KEY: [u8; 2] = [0x03, 0x79];
pub const OP_SHADOW_UP_ALL_OPS_VALUE: u64 = 50;

// Shadow down all operation weight key & value.
pub const OP_SHADOW_DOWN_ALL_OPS_KEY: [u8; 2] = [0x03, 0x7A];
pub const OP_SHADOW_DOWN_ALL_OPS_VALUE: u64 = 50;

// Shadow num allocs operation weight key & value.
pub const OP_SHADOW_NUM_ALLOCS_OPS_KEY: [u8; 2] = [0x03, 0x7B];
pub const OP_SHADOW_NUM_ALLOCS_OPS_VALUE: u64 = 1;

// Shadow allocs sum operation weight key & value.
pub const OP_SHADOW_ALLOCS_SUM_OPS_KEY: [u8; 2] = [0x03, 0x7C];
pub const OP_SHADOW_ALLOCS_SUM_OPS_VALUE: u64 = 1;

/// Coin operation weights.
/// ---------------------------------------------
// Ext balance operation weight key & value.
pub const OP_EXT_BALANCE_OPS_KEY: [u8; 2] = [0x03, 0x7D];
pub const OP_EXT_BALANCE_OPS_VALUE: u64 = 1;

// Self balance operation weight key & value.
pub const OP_SELF_BALANCE_OPS_KEY: [u8; 2] = [0x03, 0x7E];
pub const OP_SELF_BALANCE_OPS_VALUE: u64 = 1;

// Transfer operation weight key & value.
pub const OP_TRANSFER_OPS_KEY: [u8; 2] = [0x03, 0x7F];
pub const OP_TRANSFER_OPS_VALUE: u64 = 10;

/// Storage operation weights.
/// ---------------------------------------------
// Storage write operation weight key & value.
pub const OP_SWRITE_OPS_KEY: [u8; 2] = [0x03, 0x80];
pub const OP_SWRITE_OPS_VALUE: u64 = 50;

// Storage read operation weight key & value.
pub const OP_SREAD_OPS_KEY: [u8; 2] = [0x03, 0x81];
pub const OP_SREAD_OPS_VALUE: u64 = 50;

/// Memory operation weights.
/// ---------------------------------------------
// Memory write operation weight key & value.
pub const OP_MWRITE_OPS_KEY: [u8; 2] = [0x03, 0x82];
pub const OP_MWRITE_OPS_VALUE: u64 = 5;

// Memory read operation weight key & value.
pub const OP_MREAD_OPS_KEY: [u8; 2] = [0x03, 0x83];
pub const OP_MREAD_OPS_VALUE: u64 = 5;

// Memory free operation weight key & value.
pub const OP_MFREE_OPS_KEY: [u8; 2] = [0x03, 0x84];
pub const OP_MFREE_OPS_VALUE: u64 = 1;
