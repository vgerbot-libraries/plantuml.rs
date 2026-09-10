//! Eater for `!includedef` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIncludeDef`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!includedef` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIncludeDef`.
pub struct EaterIncludeDef {
    eater: Eater,
    location: Option<String>,
}

impl EaterIncludeDef {
    /// Creates a new `EaterIncludeDef`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            location: None,
        }
    }

    /// Analyzes the `!includedef` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!includedef")?;
        self.eater.skip_spaces();
        let loc = self.eater.eat_all_to_end();
        self.location = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(loc, self.eater.get_line_location()),
        );
        Ok(())
    }

    /// Returns the definition location.
    #[must_use]
    pub fn get_location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}
