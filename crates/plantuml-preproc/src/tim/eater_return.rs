//! Eater for `!return` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterReturn`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!return` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterReturn`.
pub struct EaterReturn {
    eater: Eater,
    value: Option<TValue>,
}

impl EaterReturn {
    /// Creates a new `EaterReturn`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            value: None,
        }
    }

    /// Analyzes the `!return` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!return")?;
        self.eater.skip_spaces();
        self.value = Some(self.eater.eat_expression(context, memory)?);
        Ok(())
    }

    /// Returns the return value.
    #[must_use]
    pub fn get_value2(&self) -> Option<&TValue> {
        self.value.as_ref()
    }

    /// Returns the return value (for moving).
    #[must_use]
    pub fn take_value(&mut self) -> Option<TValue> {
        self.value.take()
    }
}
