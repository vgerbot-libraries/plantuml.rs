//! Eater for `!ifndef` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIfndef`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!ifndef` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIfndef`.
pub struct EaterIfndef {
    eater: Eater,
    varname: Option<String>,
}

impl EaterIfndef {
    /// Creates a new `EaterIfndef`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            varname: None,
        }
    }

    /// Analyzes the `!ifndef` directive.
    pub fn analyze(&mut self, _context: &mut TContext, _memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!ifndef")?;
        self.eater.skip_spaces();
        self.varname = Some(self.eater.eat_and_get_varname()?);
        Ok(())
    }

    /// Returns `true` if the ifndef condition is satisfied.
    ///
    /// Ported from `EaterIfndef.isTrue`.
    #[must_use]
    pub fn is_true(&self, context: &TContext, memory: &dyn TMemory) -> bool {
        let varname = self.varname.as_deref().unwrap_or("");
        let current_value = memory.get_variable(varname);
        current_value.is_none() && !context.does_function_exist(varname)
    }
}
