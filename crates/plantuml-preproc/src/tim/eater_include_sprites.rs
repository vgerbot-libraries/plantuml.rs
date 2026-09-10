//! Eater for `!include_sprites` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterIncludeSprites`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!include_sprites` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterIncludeSprites`.
pub struct EaterIncludeSprites {
    eater: Eater,
    what: Option<String>,
}

impl EaterIncludeSprites {
    /// Creates a new `EaterIncludeSprites`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            what: None,
        }
    }

    /// Analyzes the `!include_sprites` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!include_sprites")?;
        self.eater.skip_spaces();
        let what = self.eater.eat_all_to_end();
        self.what = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(what, self.eater.get_line_location()),
        );
        Ok(())
    }

    /// Returns the include target.
    #[must_use]
    pub fn get_what(&self) -> Option<&str> {
        self.what.as_deref()
    }
}
