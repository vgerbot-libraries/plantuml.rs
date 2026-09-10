//! Eater for `!startsub` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterStartsub`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!startsub` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterStartsub`.
pub struct EaterStartsub {
    eater: Eater,
    subname: Option<String>,
}

impl EaterStartsub {
    /// Creates a new `EaterStartsub`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            subname: None,
        }
    }

    /// Analyzes the `!startsub` directive.
    pub fn analyze(&mut self, _context: &mut TContext, _memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!startsub")?;
        self.eater.skip_spaces();
        let subname = self.eater.eat_all_to_end();
        if !subname.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(EaterException::new("Bad sub name", self.eater.get_string_located()));
        }
        self.subname = Some(subname);
        Ok(())
    }

    /// Returns the sub name.
    #[must_use]
    pub fn get_subname(&self) -> Option<&str> {
        self.subname.as_deref()
    }
}
