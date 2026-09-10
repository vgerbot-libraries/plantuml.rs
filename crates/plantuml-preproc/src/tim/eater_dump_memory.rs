//! Eater for `!dump_memory` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterDumpMemory`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!dump_memory` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterDumpMemory`.
pub struct EaterDumpMemory {
    eater: Eater,
}

impl EaterDumpMemory {
    /// Creates a new `EaterDumpMemory`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self { eater: Eater::new(s) }
    }

    /// Analyzes the `!dump_memory` directive.
    pub fn analyze(&mut self, _context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!dump_memory")?;
        self.eater.skip_spaces();
        let remain = self.eater.eat_all_to_end();
        memory.dump_debug(&remain);
        Ok(())
    }
}
