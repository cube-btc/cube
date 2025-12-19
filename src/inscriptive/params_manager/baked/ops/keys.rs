/// PREFIX BYTE: 0x02.

/// Data push operation weights db key bytes.
/// ------------------------------------------------------------
pub const OP_FALSE_OPS_DB_KEY: [u8; 2] = [0x02, 0x00];

pub const OP_PUSHDATA_BASE_OPS_DB_KEY: [u8; 2] = [0x02, 0x01];

pub const OP_PUSHDATA_MULTIPLIER_OPS_DB_KEY: [u8; 2] = [0x02, 0x02];

pub const OP_TRUE_OPS_DB_KEY: [u8; 2] = [0x02, 0x03];

pub const OP_2_OPS_DB_KEY: [u8; 2] = [0x02, 0x04];

pub const OP_3_OPS_DB_KEY: [u8; 2] = [0x02, 0x05];

pub const OP_4_OPS_DB_KEY: [u8; 2] = [0x02, 0x06];

pub const OP_5_OPS_DB_KEY: [u8; 2] = [0x02, 0x07];

pub const OP_6_OPS_DB_KEY: [u8; 2] = [0x02, 0x08];

pub const OP_7_OPS_DB_KEY: [u8; 2] = [0x02, 0x09];

pub const OP_8_OPS_DB_KEY: [u8; 2] = [0x02, 0x0A];

pub const OP_9_OPS_DB_KEY: [u8; 2] = [0x02, 0x0B];

pub const OP_10_OPS_DB_KEY: [u8; 2] = [0x02, 0x0C];

pub const OP_11_OPS_DB_KEY: [u8; 2] = [0x02, 0x0D];

pub const OP_12_OPS_DB_KEY: [u8; 2] = [0x02, 0x0E];

pub const OP_13_OPS_DB_KEY: [u8; 2] = [0x02, 0x0F];

pub const OP_14_OPS_DB_KEY: [u8; 2] = [0x02, 0x10];

pub const OP_15_OPS_DB_KEY: [u8; 2] = [0x02, 0x11];

pub const OP_16_OPS_DB_KEY: [u8; 2] = [0x02, 0x12];

/// Flow control operation weights.
/// ------------------------------------------------------------
pub const OP_NOP_OPS_DB_KEY: [u8; 2] = [0x02, 0x13];

pub const OP_JUMP_OPS_DB_KEY: [u8; 2] = [0x02, 0x14];

pub const OP_IF_OPS_DB_KEY: [u8; 2] = [0x02, 0x15];

pub const OP_NOTIF_OPS_DB_KEY: [u8; 2] = [0x02, 0x16];

pub const OP_RETURNALL_OPS_DB_KEY: [u8; 2] = [0x02, 0x17];

pub const OP_RETURNSOME_OPS_DB_KEY: [u8; 2] = [0x02, 0x18];

pub const OP_ELSE_OPS_DB_KEY: [u8; 2] = [0x02, 0x19];

pub const OP_ENDIF_OPS_DB_KEY: [u8; 2] = [0x02, 0x1A];

pub const OP_VERIFY_OPS_DB_KEY: [u8; 2] = [0x02, 0x1B];

pub const OP_FAIL_OPS_DB_KEY: [u8; 2] = [0x02, 0x1C];

/// Altstack operation weights.
/// ------------------------------------------------------------
pub const OP_TOALTSTACK_OPS_DB_KEY: [u8; 2] = [0x02, 0x1D];

pub const OP_FROMALTSTACK_OPS_DB_KEY: [u8; 2] = [0x02, 0x1E];

/// Stack operation weights.
/// ------------------------------------------------------------
pub const OP_2DROP_OPS_DB_KEY: [u8; 2] = [0x02, 0x1F];

pub const OP_2DUP_OPS_DB_KEY: [u8; 2] = [0x02, 0x20];

pub const OP_3DUP_OPS_DB_KEY: [u8; 2] = [0x02, 0x21];

pub const OP_2OVER_OPS_DB_KEY: [u8; 2] = [0x02, 0x22];

pub const OP_2ROT_OPS_DB_KEY: [u8; 2] = [0x02, 0x23];

pub const OP_2SWAP_OPS_DB_KEY: [u8; 2] = [0x02, 0x24];

pub const OP_IFDUP_OPS_DB_KEY: [u8; 2] = [0x02, 0x25];

pub const OP_DEPTH_OPS_DB_KEY: [u8; 2] = [0x02, 0x26];

pub const OP_DROP_OPS_DB_KEY: [u8; 2] = [0x02, 0x27];

pub const OP_DUP_OPS_DB_KEY: [u8; 2] = [0x02, 0x28];

pub const OP_NIP_OPS_DB_KEY: [u8; 2] = [0x02, 0x29];

pub const OP_OVER_OPS_DB_KEY: [u8; 2] = [0x02, 0x2A];

pub const OP_PICK_OPS_DB_KEY: [u8; 2] = [0x02, 0x2B];

pub const OP_ROLL_OPS_DB_KEY: [u8; 2] = [0x02, 0x2C];

pub const OP_ROT_OPS_DB_KEY: [u8; 2] = [0x02, 0x2D];

pub const OP_SWAP_OPS_DB_KEY: [u8; 2] = [0x02, 0x2E];

pub const OP_TUCK_OPS_DB_KEY: [u8; 2] = [0x02, 0x2F];

/// Splice operation weights.
/// ------------------------------------------------------------
pub const OP_CAT_OPS_DB_KEY: [u8; 2] = [0x02, 0x30];

pub const OP_SPLIT_OPS_DB_KEY: [u8; 2] = [0x02, 0x31];

pub const OP_LEFT_OPS_DB_KEY: [u8; 2] = [0x02, 0x32];

pub const OP_RIGHT_OPS_DB_KEY: [u8; 2] = [0x02, 0x33];

pub const OP_SIZE_OPS_DB_KEY: [u8; 2] = [0x02, 0x34];

/// Bitwise operation weights.
/// ------------------------------------------------------------
pub const OP_INVERT_OPS_DB_KEY: [u8; 2] = [0x02, 0x35];

pub const OP_AND_OPS_DB_KEY: [u8; 2] = [0x02, 0x36];

pub const OP_OR_OPS_DB_KEY: [u8; 2] = [0x02, 0x37];

pub const OP_XOR_OPS_DB_KEY: [u8; 2] = [0x02, 0x38];

pub const OP_EQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x39];

pub const OP_EQUALVERIFY_OPS_DB_KEY: [u8; 2] = [0x02, 0x3A];

pub const OP_REVERSE_OPS_DB_KEY: [u8; 2] = [0x02, 0x3B];

/// Arithmetic operation weights.
/// ------------------------------------------------------------
pub const OP_1ADD_OPS_DB_KEY: [u8; 2] = [0x02, 0x3C];

pub const OP_1SUB_OPS_DB_KEY: [u8; 2] = [0x02, 0x3D];

pub const OP_2MUL_OPS_DB_KEY: [u8; 2] = [0x02, 0x3E];

pub const OP_2DIV_OPS_DB_KEY: [u8; 2] = [0x02, 0x3F];

pub const OP_ADDMOD_OPS_DB_KEY: [u8; 2] = [0x02, 0x40];

pub const OP_MULMOD_OPS_DB_KEY: [u8; 2] = [0x02, 0x41];

pub const OP_NOT_OPS_DB_KEY: [u8; 2] = [0x02, 0x42];

pub const OP_0NOTEQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x43];

pub const OP_ADD_OPS_DB_KEY: [u8; 2] = [0x02, 0x44];

pub const OP_SUB_OPS_DB_KEY: [u8; 2] = [0x02, 0x45];

pub const OP_MUL_OPS_DB_KEY: [u8; 2] = [0x02, 0x46];

pub const OP_DIV_OPS_DB_KEY: [u8; 2] = [0x02, 0x47];

pub const OP_LSHIFT_OPS_DB_KEY: [u8; 2] = [0x02, 0x48];

pub const OP_RSHIFT_OPS_DB_KEY: [u8; 2] = [0x02, 0x49];

pub const OP_BOOLAND_OPS_DB_KEY: [u8; 2] = [0x02, 0x4A];

pub const OP_BOOLOR_OPS_DB_KEY: [u8; 2] = [0x02, 0x4B];

pub const OP_NUMEQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x4C];

pub const OP_NUMEQUALVERIFY_OPS_DB_KEY: [u8; 2] = [0x02, 0x4D];

pub const OP_NUMNOTEQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x4E];

pub const OP_LESSTHAN_OPS_DB_KEY: [u8; 2] = [0x02, 0x4F];

pub const OP_GREATERTHAN_OPS_DB_KEY: [u8; 2] = [0x02, 0x50];

pub const OP_LESSTHANOREQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x51];

pub const OP_GREATERTHANOREQUAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x52];

pub const OP_MIN_OPS_DB_KEY: [u8; 2] = [0x02, 0x53];

pub const OP_MAX_OPS_DB_KEY: [u8; 2] = [0x02, 0x54];

pub const OP_WITHIN_OPS_DB_KEY: [u8; 2] = [0x02, 0x55];

/// Digest operation weights.
/// ------------------------------------------------------------
pub const OP_RIPEMD160_OPS_DB_KEY: [u8; 2] = [0x02, 0x56];

pub const OP_SHA1_OPS_DB_KEY: [u8; 2] = [0x02, 0x57];

pub const OP_SHA256_OPS_DB_KEY: [u8; 2] = [0x02, 0x58];

pub const OP_HASH160_OPS_DB_KEY: [u8; 2] = [0x02, 0x59];

pub const OP_HASH256_OPS_DB_KEY: [u8; 2] = [0x02, 0x5A];

pub const OP_TAGGEDHASH_OPS_DB_KEY: [u8; 2] = [0x02, 0x5B];

pub const OP_BLAKE2BVAR_BASE_OPS_DB_KEY: [u8; 2] = [0x02, 0x5C];
pub const OP_BLAKE2BVAR_MULTIPLIER_OPS_DB_KEY: [u8; 2] = [0x02, 0x5D];

pub const OP_BLAKE2SVAR_BASE_OPS_DB_KEY: [u8; 2] = [0x02, 0x5E];
pub const OP_BLAKE2SVAR_MULTIPLIER_OPS_DB_KEY: [u8; 2] = [0x02, 0x5F];

/// Secp operation weights.
/// ------------------------------------------------------------
pub const OP_SECPSCALARADD_OPS_DB_KEY: [u8; 2] = [0x02, 0x60];

pub const OP_SECPSCALARMUL_OPS_DB_KEY: [u8; 2] = [0x02, 0x61];

pub const OP_SECPPOINTADD_OPS_DB_KEY: [u8; 2] = [0x02, 0x62];

pub const OP_SECPPOINTMUL_OPS_DB_KEY: [u8; 2] = [0x02, 0x63];

pub const OP_PUSHSECPGENERATORPOINT_OPS_DB_KEY: [u8; 2] = [0x02, 0x64];

pub const OP_ISZEROSECPSCALAR_OPS_DB_KEY: [u8; 2] = [0x02, 0x65];

pub const OP_ISINFINITESECPPOINT_OPS_DB_KEY: [u8; 2] = [0x02, 0x66];

/// Digital signature operation weights.
/// ------------------------------------------------------------
pub const OP_CHECKSCHNORRSIG_OPS_DB_KEY: [u8; 2] = [0x02, 0x67];

pub const OP_CHECKSCHNORRSIGBIP340_OPS_DB_KEY: [u8; 2] = [0x02, 0x68];

pub const OP_CHECKBLSSIG_OPS_DB_KEY: [u8; 2] = [0x02, 0x69];

pub const OP_CHECKBLSSIGAGG_BASE_OPS_DB_KEY: [u8; 2] = [0x02, 0x6A];
pub const OP_CHECKBLSSIGAGG_MULTIPLIER_OPS_DB_KEY: [u8; 2] = [0x02, 0x6B];

/// Call info operation weights.
/// ------------------------------------------------------------
pub const OP_CALLER_OPS_DB_KEY: [u8; 2] = [0x02, 0x6C];

pub const OP_OPSBUDGET_OPS_DB_KEY: [u8; 2] = [0x02, 0x6D];

pub const OP_OPSCOUNTER_OPS_DB_KEY: [u8; 2] = [0x02, 0x6E];

pub const OP_OPSPRICE_OPS_DB_KEY: [u8; 2] = [0x02, 0x6F];

pub const OP_TIMESTAMP_OPS_DB_KEY: [u8; 2] = [0x02, 0x70];

/// Call operation weights.
/// ------------------------------------------------------------
pub const OP_CALL_OPS_DB_KEY: [u8; 2] = [0x02, 0x71];

pub const OP_CALLEXT_OPS_DB_KEY: [u8; 2] = [0x02, 0x72];

/// Shadowing operation weights.
/// ------------------------------------------------------------
pub const OP_SHADOW_ALLOC_OPS_DB_KEY: [u8; 2] = [0x02, 0x73];

pub const OP_SHADOW_DEALLOC_OPS_DB_KEY: [u8; 2] = [0x02, 0x74];

pub const OP_SHADOW_HAS_ALLOC_OPS_DB_KEY: [u8; 2] = [0x02, 0x75];

pub const OP_SHADOW_ALLOC_VAL_OPS_DB_KEY: [u8; 2] = [0x02, 0x76];

pub const OP_SHADOW_UP_OPS_DB_KEY: [u8; 2] = [0x02, 0x77];

pub const OP_SHADOW_DOWN_OPS_DB_KEY: [u8; 2] = [0x02, 0x78];

pub const OP_SHADOW_UP_ALL_OPS_DB_KEY: [u8; 2] = [0x02, 0x79];

pub const OP_SHADOW_DOWN_ALL_OPS_DB_KEY: [u8; 2] = [0x02, 0x7A];

pub const OP_SHADOW_NUM_ALLOCS_OPS_DB_KEY: [u8; 2] = [0x02, 0x7B];

pub const OP_SHADOW_ALLOCS_SUM_OPS_DB_KEY: [u8; 2] = [0x02, 0x7C];

/// Coin operation weights.
/// ------------------------------------------------------------
pub const OP_EXT_BALANCE_OPS_DB_KEY: [u8; 2] = [0x02, 0x7D];

pub const OP_SELF_BALANCE_OPS_DB_KEY: [u8; 2] = [0x02, 0x7E];

pub const OP_TRANSFER_OPS_DB_KEY: [u8; 2] = [0x02, 0x7F];

/// Storage operation weights.
/// ------------------------------------------------------------
pub const OP_SWRITE_OPS_DB_KEY: [u8; 2] = [0x02, 0x80];

pub const OP_SREAD_OPS_DB_KEY: [u8; 2] = [0x02, 0x81];

/// Memory operation weights.
/// ------------------------------------------------------------
pub const OP_MWRITE_OPS_DB_KEY: [u8; 2] = [0x02, 0x82];

pub const OP_MREAD_OPS_DB_KEY: [u8; 2] = [0x02, 0x83];

pub const OP_MFREE_OPS_DB_KEY: [u8; 2] = [0x02, 0x84];
