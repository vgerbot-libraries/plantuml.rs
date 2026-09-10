//! Execution context for `!while` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.ExecutionContextWhile`.

use super::eater_exception::EaterException;
use super::expression::TokenStack;
use super::iterator::code_position::CodePosition;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Tracks the state of a `!while` loop.
///
/// Ported from `net.sourceforge.plantuml.tim.ExecutionContextWhile`.
#[derive(Debug, Clone)]
pub struct ExecutionContextWhile {
    while_expression: TokenStack,
    code_position: Box<dyn CodePosition>,
    skip_me: bool,
}

impl ExecutionContextWhile {
    /// Creates a new `ExecutionContextWhile` from a token stack and code position.
    ///
    /// Ported from `ExecutionContextWhile.fromValue`.
    #[must_use]
    pub fn from_value(while_expression: TokenStack, code_position: Box<dyn CodePosition>) -> Self {
        Self {
            while_expression,
            code_position,
            skip_me: false,
        }
    }

    /// Evaluates the while condition.
    ///
    /// Ported from `ExecutionContextWhile.conditionValue`.
    pub fn condition_value(
        &self,
        location: &StringLocated,
        context: &mut TContext,
        memory: &mut dyn TMemory,
    ) -> Result<super::expression::TValue, EaterException> {
        self.while_expression.get_result(location, context, memory)
    }

    /// Marks this while loop to be skipped.
    pub fn skip_me(&mut self) {
        self.skip_me = true;
    }

    /// Returns `true` if this while loop should be skipped.
    #[must_use]
    pub fn is_skip_me(&self) -> bool {
        self.skip_me
    }

    /// Returns the code position where the while loop starts.
    #[must_use]
    pub fn get_start_while(&self) -> &dyn CodePosition {
        self.code_position.as_ref()
    }
}
