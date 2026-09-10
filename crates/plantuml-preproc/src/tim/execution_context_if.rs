//! Execution context for `!if` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.ExecutionContextIf`.

/// Tracks the state of an `!if` conditional block.
///
/// Ported from `net.sourceforge.plantuml.tim.ExecutionContextIf`.
#[derive(Debug, Clone)]
pub struct ExecutionContextIf {
    is_true: bool,
    has_been_burn: bool,
}

impl ExecutionContextIf {
    /// Creates a new `ExecutionContextIf` from a boolean value.
    ///
    /// Ported from `ExecutionContextIf.fromValue`.
    #[must_use]
    pub fn from_value(is_true: bool) -> Self {
        let has_been_burn = is_true;
        Self {
            is_true,
            has_been_burn,
        }
    }

    /// Returns `true` if the condition is satisfied here.
    #[must_use]
    pub fn condition_is_ok_here(&self) -> bool {
        self.is_true
    }

    /// Entering an `!elseif` — set condition to false.
    pub fn entering_else_if(&mut self) {
        self.is_true = false;
    }

    /// Now in the `!else` block.
    pub fn now_in_else(&mut self) {
        self.is_true = !self.has_been_burn;
    }

    /// Now in some `!elseif` that is true.
    pub fn now_in_some_else_if(&mut self) {
        self.is_true = true;
        self.has_been_burn = true;
    }

    /// Returns `true` if this if block has been "burned" (executed).
    #[must_use]
    pub fn has_been_burn(&self) -> bool {
        self.has_been_burn
    }

    /// Sets the burned flag.
    pub fn set_has_been_burn(&mut self, has_been_burn: bool) {
        self.has_been_burn = has_been_burn;
    }
}
