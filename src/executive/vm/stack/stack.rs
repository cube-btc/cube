use super::{
    limits::{MAX_STACK_ITEMS_COUNT, MAX_STACK_ITEM_SIZE},
    stack_error::StackError,
    stack_item::StackItem,
};
use std::fmt;

/// The stack newtype wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack(pub Vec<StackItem>);

impl Stack {
    /// Creates a new stack.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a new stack with the given items.
    pub fn new_with_items(items: Vec<StackItem>) -> Self {
        Self(items)
    }

    /// Returns the length of the stack.
    pub fn items_count(&self) -> u32 {
        self.0.len() as u32
    }

    /// Returns the items of the stack.
    pub fn items(&self) -> Vec<StackItem> {
        self.0.clone()
    }

    /// Pushes a stack item to the stack.
    pub fn push(&mut self, item: StackItem) -> Result<(), StackError> {
        if item.len() > MAX_STACK_ITEM_SIZE {
            return Err(StackError::StackItemTooLarge);
        }

        if self.items_count() >= MAX_STACK_ITEMS_COUNT {
            return Err(StackError::StackTooLarge);
        }
        self.0.push(item);

        Ok(())
    }

    /// Pops a stack item from the stack.
    pub fn pop(&mut self) -> Result<StackItem, StackError> {
        self.0.pop().ok_or(StackError::EmptyStack)
    }

    /// Clones and returns the last item from the stack.
    pub fn last_item(&self) -> Result<StackItem, StackError> {
        self.0.last().cloned().ok_or(StackError::EmptyStack)
    }
}
impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for (i, item) in self.0.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, item)?;
        }

        Ok(())
    }
}
