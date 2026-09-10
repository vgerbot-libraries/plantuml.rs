//! Eater for `!while` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterWhile`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::TokenStack;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!while` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterWhile`.
pub struct EaterWhile {
    eater: Eater,
    expression: Option<TokenStack>,
}

impl EaterWhile {
    /// Creates a new `EaterWhile`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            expression: None,
        }
    }

    /// Analyzes the `!while` directive.
    pub fn analyze(&mut self, _context: &mut TContext, _memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!while")?;
        self.eater.skip_spaces();
        self.expression = Some(self.eater.eat_token_stack()?);
        Ok(())
    }

    /// Returns the while expression token stack.
    #[must_use]
    pub fn get_while_expression(&self) -> Option<&TokenStack> {
        self.expression.as_ref()
    }

    /// Returns the while expression token stack (for moving).
    #[must_use]
    pub fn take_while_expression(&mut self) -> Option<TokenStack> {
        self.expression.take()
    }
}
