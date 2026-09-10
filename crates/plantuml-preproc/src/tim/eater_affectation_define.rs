//! Eater for `!define` variable directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterAffectationDefine`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_memory::TMemory;
use super::t_variable_scope::TVariableScope;
use crate::StringLocated;

/// Parses `!define` variable directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterAffectationDefine`.
pub struct EaterAffectationDefine {
    eater: Eater,
}

impl EaterAffectationDefine {
    /// Creates a new `EaterAffectationDefine`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s.get_trimmed()),
        }
    }

    /// Analyzes the `!define` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!define")?;
        self.eater.skip_spaces();
        let varname = self.eater.eat_and_get_varname()?;
        self.eater.skip_spaces();
        let tmp = self.eater.eat_all_to_end();
        let tmp2 = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(tmp, self.eater.get_line_location()),
        );
        let value = TValue::from_string(tmp2.unwrap_or_default());
        memory.put_variable(&varname, value, Some(TVariableScope::Global), self.eater.get_string_located())?;
        Ok(())
    }
}
