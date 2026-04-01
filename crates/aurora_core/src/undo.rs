//! Generic undo/redo stack.
//!
//! Stores state snapshots and provides undo/redo operations. Push a
//! snapshot before each mutation; undo restores the previous state;
//! redo reverses an undo. Any new push after an undo clears the redo
//! stack (branching history is not supported).
//!
//! # Example
//!
//! ```
//! use aurora_core::undo::UndoStack;
//!
//! let mut stack = UndoStack::new();
//! let mut value = "hello".to_string();
//!
//! // Save state before mutation
//! stack.push(value.clone());
//! value.push_str(" world");
//!
//! // Undo
//! if let Some(prev) = stack.undo(value.clone()) {
//!     value = prev;
//! }
//! assert_eq!(value, "hello");
//!
//! // Redo
//! if let Some(next) = stack.redo(value.clone()) {
//!     value = next;
//! }
//! assert_eq!(value, "hello world");
//! ```

const DEFAULT_MAX_DEPTH: usize = 100;

/// A generic undo/redo stack that stores state snapshots.
pub struct UndoStack<S> {
    undo: Vec<S>,
    redo: Vec<S>,
    max_depth: usize,
}

impl<S> UndoStack<S> {
    /// Creates a new undo stack with the default max depth (100).
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Creates a new undo stack with the given max depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_depth,
        }
    }

    /// Pushes a snapshot onto the undo stack. Clears the redo stack
    /// (any previously undone states are discarded).
    ///
    /// If the stack exceeds `max_depth`, the oldest entry is removed.
    pub fn push(&mut self, state: S) {
        self.redo.clear();
        self.undo.push(state);
        if self.undo.len() > self.max_depth {
            self.undo.remove(0);
        }
    }

    /// Undoes the last action. Returns the previous state, or `None` if
    /// the undo stack is empty.
    ///
    /// The caller must provide the current state, which is pushed onto
    /// the redo stack so it can be restored later.
    pub fn undo(&mut self, current: S) -> Option<S> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    /// Redoes the last undone action. Returns the state to restore, or
    /// `None` if the redo stack is empty.
    ///
    /// The caller must provide the current state, which is pushed onto
    /// the undo stack.
    pub fn redo(&mut self, current: S) -> Option<S> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }

    /// Returns `true` if there are states to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Returns `true` if there are states to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Clears both the undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

impl<S> Default for UndoStack<S> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_undo() {
        let mut stack = UndoStack::new();
        stack.push("a".to_string());
        let prev = stack.undo("b".to_string());
        assert_eq!(prev, Some("a".to_string()));
    }

    #[test]
    fn undo_redo_cycle() {
        let mut stack = UndoStack::new();
        stack.push("before".to_string());
        // Undo: restore "before", save "after" to redo
        let prev = stack.undo("after".to_string()).unwrap();
        assert_eq!(prev, "before");
        // Redo: restore "after", save "before" to undo
        let next = stack.redo("before".to_string()).unwrap();
        assert_eq!(next, "after");
    }

    #[test]
    fn push_after_undo_clears_redo() {
        let mut stack = UndoStack::new();
        stack.push("a".to_string());
        stack.push("b".to_string());
        // Undo once
        let _ = stack.undo("c".to_string());
        assert!(stack.can_redo());
        // New push clears redo
        stack.push("d".to_string());
        assert!(!stack.can_redo());
    }

    #[test]
    fn max_depth_enforced() {
        let mut stack = UndoStack::with_max_depth(3);
        stack.push("a".to_string());
        stack.push("b".to_string());
        stack.push("c".to_string());
        stack.push("d".to_string()); // "a" should be dropped
        assert_eq!(stack.undo("e".to_string()), Some("d".to_string()));
        assert_eq!(stack.undo("e".to_string()), Some("c".to_string()));
        assert_eq!(stack.undo("e".to_string()), Some("b".to_string()));
        assert_eq!(stack.undo("e".to_string()), None); // "a" was dropped
    }

    #[test]
    fn empty_undo_returns_none() {
        let mut stack: UndoStack<String> = UndoStack::new();
        assert_eq!(stack.undo("current".to_string()), None);
    }

    #[test]
    fn empty_redo_returns_none() {
        let mut stack: UndoStack<String> = UndoStack::new();
        assert_eq!(stack.redo("current".to_string()), None);
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut stack = UndoStack::new();
        stack.push("a".to_string());
        let _ = stack.undo("b".to_string());
        assert!(stack.can_redo());
        stack.clear();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn multiple_undo_redo() {
        let mut stack = UndoStack::new();
        stack.push("a".to_string());
        stack.push("b".to_string());
        stack.push("c".to_string());
        // Undo 3 times
        assert_eq!(stack.undo("d".to_string()), Some("c".to_string()));
        assert_eq!(stack.undo("c".to_string()), Some("b".to_string()));
        assert_eq!(stack.undo("b".to_string()), Some("a".to_string()));
        assert_eq!(stack.undo("a".to_string()), None);
        // Redo 3 times
        assert_eq!(stack.redo("a".to_string()), Some("b".to_string()));
        assert_eq!(stack.redo("b".to_string()), Some("c".to_string()));
        assert_eq!(stack.redo("c".to_string()), Some("d".to_string()));
        assert_eq!(stack.redo("d".to_string()), None);
    }
}
