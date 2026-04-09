// Data push
pub const OP_FALSE_OPS: u32 = 1;
pub const OP_TRUE_OPS: u32 = 1;
pub const OP_2_OPS: u32 = 1;
pub const OP_3_OPS: u32 = 1;
pub const OP_4_OPS: u32 = 1;
pub const OP_5_OPS: u32 = 1;
pub const OP_6_OPS: u32 = 1;
pub const OP_7_OPS: u32 = 1;
pub const OP_8_OPS: u32 = 1;
pub const OP_9_OPS: u32 = 1;
pub const OP_10_OPS: u32 = 1;
pub const OP_11_OPS: u32 = 1;
pub const OP_12_OPS: u32 = 1;
pub const OP_13_OPS: u32 = 1;
pub const OP_14_OPS: u32 = 1;
pub const OP_15_OPS: u32 = 1;
pub const OP_16_OPS: u32 = 1;

// Flow
pub const OP_NOP_OPS: u32 = 1;
pub const OP_RETURNERR_OPS: u32 = 1;
pub const OP_IF_OPS: u32 = 1;
pub const OP_NOTIF_OPS: u32 = 1;
pub const OP_ELSE_OPS: u32 = 1;
pub const OP_ENDIF_OPS: u32 = 1;
pub const OP_VERIFY_OPS: u32 = 1;
pub const OP_RETURNALL_OPS: u32 = 1;
pub const OP_RETURNSOME_OPS: u32 = 1;
pub const OP_FAIL_OPS: u32 = 1;

// Alt stack
pub const OP_FROMALTSTACK_OPS: u32 = 1;
pub const OP_TOALTSTACK_OPS: u32 = 1;

// Stack operations
pub const OP_2DROP_OPS: u32 = 2;
pub const OP_2DUP_OPS: u32 = 2;
pub const OP_3DUP_OPS: u32 = 3;
pub const OP_2OVER_OPS: u32 = 2;
pub const OP_2ROT_OPS: u32 = 2;
pub const OP_2SWAP_OPS: u32 = 2;
pub const OP_IFDUP_OPS: u32 = 1;
pub const OP_DEPTH_OPS: u32 = 1;
pub const OP_DROP_OPS: u32 = 1;
pub const OP_DUP_OPS: u32 = 1;
pub const OP_NIP_OPS: u32 = 1;
pub const OP_OVER_OPS: u32 = 1;
pub const OP_PICK_OPS: u32 = 1;
pub const OP_ROLL_OPS: u32 = 1;
pub const OP_ROT_OPS: u32 = 1;
pub const OP_SWAP_OPS: u32 = 1;
pub const OP_TUCK_OPS: u32 = 1;

// Splice
pub const OP_CAT_OPS: u32 = 2;
pub const OP_SPLIT_OPS: u32 = 2;
pub const OP_LEFT_OPS: u32 = 2;
pub const OP_RIGHT_OPS: u32 = 2;
pub const OP_SIZE_OPS: u32 = 1;

// Bitwise
pub const OP_INVERT_OPS: u32 = 2;
pub const OP_AND_OPS: u32 = 2;
pub const OP_OR_OPS: u32 = 2;
pub const OP_XOR_OPS: u32 = 2;
pub const OP_EQUAL_OPS: u32 = 1;
pub const OP_EQUALVERIFY_OPS: u32 = 2;
pub const OP_REVERSE_OPS: u32 = 3;

// Arithmetic
pub const OP_1ADD_OPS: u32 = 3;
pub const OP_1SUB_OPS: u32 = 3;
pub const OP_2MUL_OPS: u32 = 5;
pub const OP_2DIV_OPS: u32 = 5;
pub const OP_ADDMOD_OPS: u32 = 3;
pub const OP_MULMOD_OPS: u32 = 3;
pub const OP_NOT_OPS: u32 = 1;
pub const OP_0NOTEQUAL_OPS: u32 = 1;
pub const OP_ADD_OPS: u32 = 3;
pub const OP_SUB_OPS: u32 = 3;
pub const OP_MUL_OPS: u32 = 5;
pub const OP_DIV_OPS: u32 = 5;
pub const OP_LSHIFT_OPS: u32 = 3;
pub const OP_RSHIFT_OPS: u32 = 3;
pub const OP_BOOLAND_OPS: u32 = 2;
pub const OP_BOOLOR_OPS: u32 = 2;
pub const OP_NUMEQUAL_OPS: u32 = 1;
pub const OP_NUMEQUALVERIFY_OPS: u32 = 2;
pub const OP_NUMNOTEQUAL_OPS: u32 = 1;
pub const OP_LESSTHAN_OPS: u32 = 1;
pub const OP_GREATERTHAN_OPS: u32 = 1;
pub const OP_LESSTHANOREQUAL_OPS: u32 = 1;
pub const OP_GREATERTHANOREQUAL_OPS: u32 = 1;
pub const OP_MIN_OPS: u32 = 1;
pub const OP_MAX_OPS: u32 = 1;
pub const OP_WITHIN_OPS: u32 = 1;

// Crypto

// Memory
pub const OP_MREAD_OPS: u32 = 5;
pub const OP_MWRITE_OPS: u32 = 5;
pub const OP_MFREE_OPS: u32 = 1;
