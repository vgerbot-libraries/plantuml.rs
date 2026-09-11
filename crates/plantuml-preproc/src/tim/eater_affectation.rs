//! Eater for `!local`/`!global` variable affectation.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterAffectation`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use super::t_variable_scope::TVariableScope;
use crate::StringLocated;

/// Parses variable affectation directives (`!$var = value`).
///
/// Ported from `net.sourceforge.plantuml.tim.EaterAffectation`.
pub struct EaterAffectation {
    eater: Eater,
}

impl EaterAffectation {
    /// Creates a new `EaterAffectation`.
    #[must_use]
    pub fn new(sl: StringLocated) -> Self {
        Self {
            eater: Eater::new(sl.get_trimmed()),
        }
    }

    /// Analyzes the affectation directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_char('!')?;
        self.eater.skip_spaces();
        let mut varname = self.eater.eat_and_get_varname()?;
        let mut scope = TVariableScope::lazzy_parse(&varname);
        if scope.is_some() {
            self.eater.skip_spaces();
            let ch = self.eater.peek_char();
            if ch == '?' || ch == '=' {
                scope = None;
            } else {
                varname = self.eater.eat_and_get_varname()?;
            }
        }
        self.eater.skip_spaces();
        let conditional = if self.eater.peek_char() == '?' {
            self.eater.check_and_eat_char('?')?;
            true
        } else {
            false
        };
        self.eater.check_and_eat_char('=')?;
        if conditional
            && memory.get_variable(&varname).is_some() {
                return Ok(());
            }
        self.eater.skip_spaces();
        let value = self.eater.eat_expression(context, memory)?;
        memory.put_variable(&varname, value, scope, self.eater.get_string_located())?;
        Ok(())
    }
}
