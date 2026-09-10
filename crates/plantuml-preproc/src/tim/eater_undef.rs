//! Eater for `!undef` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterUndef`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!undef` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterUndef`.
pub struct EaterUndef {
    eater: Eater,
}

impl EaterUndef {
    /// Creates a new `EaterUndef`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s.get_trimmed()),
        }
    }

    /// Analyzes the `!undef` directive.
    pub fn analyze(&mut self, _context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!undef")?;
        self.eater.skip_spaces();
        let varname = self.eater.eat_and_get_varname()?;
        memory.remove_variable(&varname);
        Ok(())
    }
}
