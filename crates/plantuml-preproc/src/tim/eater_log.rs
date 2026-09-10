//! Eater for `!log` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterLog`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::stubs::Log;
use crate::StringLocated;

/// Parses `!log` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterLog`.
pub struct EaterLog {
    eater: Eater,
}

impl EaterLog {
    /// Creates a new `EaterLog`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self { eater: Eater::new(s) }
    }

    /// Analyzes the `!log` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!log")?;
        self.eater.skip_spaces();
        let log_data = self.eater.eat_all_to_end();
        let log_data = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(log_data, self.eater.get_line_location()),
        );
        if let Some(log_data) = log_data {
            Log::user_log(|| format!("[Log] {log_data}"));
        }
        Ok(())
    }
}
