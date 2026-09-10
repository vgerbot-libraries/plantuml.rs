//! Eater for `!procedure` declarations.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterDeclareProcedure`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_function_impl::TFunctionImpl;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!procedure` declarations.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterDeclareProcedure`.
pub struct EaterDeclareProcedure {
    eater: Eater,
    location: StringLocated,
    function: Option<TFunctionImpl>,
    final_flag: bool,
}

impl EaterDeclareProcedure {
    /// Creates a new `EaterDeclareProcedure`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        let location = s.clone();
        Self {
            eater: Eater::new(s.get_trimmed()),
            location,
            function: None,
            final_flag: false,
        }
    }

    /// Analyzes the `!procedure` declaration.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_char('!')?;
        let mut unquoted = false;
        while self.peek_unquoted() || self.peek_final() {
            if self.peek_unquoted() {
                self.eater.check_and_eat_str("unquoted")?;
                self.eater.skip_spaces();
                unquoted = true;
            } else if self.peek_final() {
                self.eater.check_and_eat_str("final")?;
                self.eater.skip_spaces();
                self.final_flag = true;
            }
        }
        self.eater.check_and_eat_str("procedure")?;
        self.eater.skip_spaces();
        self.function = Some(self.eater.eat_declare_procedure(context, memory, unquoted, &self.location)?);
        Ok(())
    }

    fn peek_unquoted(&self) -> bool {
        self.eater.peek_char() == 'u'
    }

    fn peek_final(&self) -> bool {
        self.eater.peek_char() == 'f' && self.eater.peek_char_n2() == 'i'
    }

    /// Returns the declared function.
    #[must_use]
    pub fn get_function(&self) -> Option<&TFunctionImpl> {
        self.function.as_ref()
    }

    /// Returns the declared function (for moving).
    #[must_use]
    pub fn take_function(&mut self) -> Option<TFunctionImpl> {
        self.function.take()
    }

    /// Returns the final flag.
    #[must_use]
    pub fn get_final_flag(&self) -> bool {
        self.final_flag
    }
}
