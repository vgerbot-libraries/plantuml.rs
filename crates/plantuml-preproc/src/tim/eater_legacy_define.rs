//! Eater for `!define` (legacy) directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterLegacyDefine`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_function_impl::TFunctionImpl;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses legacy `!define` function directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterLegacyDefine`.
pub struct EaterLegacyDefine {
    eater: Eater,
    function: Option<TFunctionImpl>,
}

impl EaterLegacyDefine {
    /// Creates a new `EaterLegacyDefine`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s.get_trimmed()),
            function: None,
        }
    }

    /// Analyzes the legacy `!define` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!define")?;
        self.eater.skip_spaces();
        let location = self.eater.get_string_located().clone();
        let mut function = self.eater.eat_declare_function(
            context,
            memory,
            true,
            &location,
            false,
            TFunctionType::LegacyDefine,
        )?;
        let def = self.eater.eat_all_to_end();
        function.set_legacy_definition(def);
        self.function = Some(function);
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
