//! Eater for `!ifdef` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIfdef`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!ifdef` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIfdef`.
pub struct EaterIfdef {
    eater: Eater,
    expression: Option<String>,
}

impl EaterIfdef {
    /// Creates a new `EaterIfdef`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            expression: None,
        }
    }

    /// Analyzes the `!ifdef` directive.
    pub fn analyze(&mut self, _context: &mut TContext, _memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!ifdef")?;
        self.eater.skip_spaces();
        self.expression = Some(self.eater.eat_all_to_end());
        Ok(())
    }

    /// Returns `true` if the ifdef condition is satisfied.
    ///
    /// Ported from `EaterIfdef.isTrue`.
    #[must_use]
    pub fn is_true(&self, context: &TContext, memory: &dyn TMemory) -> bool {
        // Simplified: check if any variable in the expression is defined
        let expr = self.expression.as_deref().unwrap_or("");
        let varname = expr.trim();
        let current_value = memory.get_variable(varname);
        current_value.is_some() || context.does_function_exist(varname)
    }
}
