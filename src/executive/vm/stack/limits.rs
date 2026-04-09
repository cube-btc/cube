/// The maximum number of items in the stack.
pub const MAX_STACK_ITEMS_COUNT: u32 = 1024 * 4;

/// The maximum size of an item in the stack.
pub const MAX_STACK_ITEM_SIZE: u32 = 1024 * 32;

/// The minimum length of a memory/storage key.
pub const MIN_KEY_LENGTH: u32 = 1;

/// The maximum length of a memory/storage key.
pub const MAX_KEY_LENGTH: u32 = 40;

/// The minimum length of a memory/storage value.
pub const MIN_VALUE_LENGTH: u32 = 1;

/// The maximum byte size of a contract memory.
pub const MAX_CONTRACT_MEMORY_SIZE: u32 = 65_536;

// Ops upper bound.
pub const OPS_LIMIT: u32 = 100_000;
