//! Eater for `!assert` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterAssert`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!assert` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterAssert`.
pub struct EaterAssert {
    eater: Eater,
}

impl EaterAssert {
    /// Creates a new `EaterAssert`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self { eater: Eater::new(s) }
    }

    /// Analyzes the `!assert` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!assert")?;
        self.eater.skip_spaces();
        let value = self.eater.eat_expression_stop_at_colon(context, memory)?;
        self.eater.skip_spaces();
        if !value.to_boolean() {
            let ch = self.eater.peek_char();
            if ch == ':' {
                self.eater.check_and_eat_char(':')?;
                let message = self.eater.eat_expression(context, memory)?;
                return Err(EaterException::new(
                    format!("Assertion error : {message}"),
                    self.eater.get_string_located(),
                ));
            }
            return Err(EaterException::new("Assertion error", self.eater.get_string_located()));
        }
        Ok(())
    }
}
