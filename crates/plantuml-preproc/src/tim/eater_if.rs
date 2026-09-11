//! Eater for `!if` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIf`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!if` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIf`.
pub struct EaterIf {
    eater: Eater,
    boolean_value: bool,
}

impl EaterIf {
    /// Creates a new `EaterIf`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            boolean_value: false,
        }
    }

    /// Analyzes the `!if` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!if")?;
        self.eater.skip_spaces();
        let value = self.eater.eat_expression(context, memory)?;
        self.boolean_value = value.to_boolean();
        Ok(())
    }

    /// Returns `true` if the condition is satisfied.
    #[must_use]
    pub fn is_true(&self) -> bool {
        self.boolean_value
    }
}
