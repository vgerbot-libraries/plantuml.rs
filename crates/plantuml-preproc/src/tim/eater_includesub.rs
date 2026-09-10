//! Eater for `!includesub` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIncludesub`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!includesub` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIncludesub`.
pub struct EaterIncludesub {
    eater: Eater,
    what: Option<String>,
}

impl EaterIncludesub {
    /// Creates a new `EaterIncludesub`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            what: None,
        }
    }

    /// Analyzes the `!includesub` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!includesub")?;
        self.eater.skip_spaces();
        let what = self.eater.eat_all_to_end();
        self.what = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(what, self.eater.get_line_location()),
        );
        Ok(())
    }

    /// Returns the sub name.
    #[must_use]
    pub fn get_what(&self) -> Option<&str> {
        self.what.as_deref()
    }
}
