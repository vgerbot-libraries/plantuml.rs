//! Eater for `!include` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterInclude`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::preproc2::PreprocessorIncludeStrategy;
use crate::StringLocated;

/// Parses `!include` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterInclude`.
pub struct EaterInclude {
    eater: Eater,
    what: Option<String>,
    strategy: PreprocessorIncludeStrategy,
}

impl EaterInclude {
    /// Creates a new `EaterInclude`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            what: None,
            strategy: PreprocessorIncludeStrategy::Default,
        }
    }

    /// Analyzes the `!include` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!include")?;
        let peek_char = self.eater.peek_char();
        if peek_char == 'u' {
            self.eater.check_and_eat_str("url")?;
        } else if peek_char == '_' {
            self.eater.check_and_eat_char('_')?;
            let peek_char2 = self.eater.peek_char();
            if peek_char2 == 'm' {
                self.eater.check_and_eat_str("many")?;
                self.strategy = PreprocessorIncludeStrategy::Many;
            } else {
                self.eater.check_and_eat_str("once")?;
                self.strategy = PreprocessorIncludeStrategy::Once;
            }
        }
        self.eater.skip_spaces();
        let what = self.eater.eat_all_to_end();
        self.what = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(what, self.eater.get_line_location()),
        );
        Ok(())
    }

    /// Returns the include target.
    #[must_use]
    pub fn get_what(&self) -> Option<&str> {
        self.what.as_deref()
    }

    /// Returns the include strategy.
    #[must_use]
    pub fn get_preprocessor_include_strategy(&self) -> &PreprocessorIncludeStrategy {
        &self.strategy
    }
}
