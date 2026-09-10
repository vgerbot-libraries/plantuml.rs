//! Eater for `!definelong` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterLegacyDefineLong`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_function_impl::TFunctionImpl;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!definelong` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterLegacyDefineLong`.
pub struct EaterLegacyDefineLong {
    eater: Eater,
    function: Option<TFunctionImpl>,
}

impl EaterLegacyDefineLong {
    /// Creates a new `EaterLegacyDefineLong`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s.get_trimmed()),
            function: None,
        }
    }

    /// Analyzes the `!definelong` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!definelong")?;
        self.eater.skip_spaces();
        let location = self.eater.get_string_located().clone();
        self.function = Some(self.eater.eat_declare_function(
            context,
            memory,
            true,
            &location,
            true,
            TFunctionType::LegacyDefineLong,
        )?);
        Ok(())
    }

    /// Returns the function.
    #[must_use]
    pub fn get_function(&self) -> Option<&TFunctionImpl> {
        self.function.as_ref()
    }

    /// Returns the function (for moving).
    #[must_use]
    pub fn take_function(&mut self) -> Option<TFunctionImpl> {
        self.function.take()
    }
}
