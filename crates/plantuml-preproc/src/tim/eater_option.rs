//! Eater for `!option` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterOption`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::preproc::OptionKey;
use crate::stubs::Warning;
use crate::stubs::WarningHandler;
use crate::StringLocated;

/// Parses `!option` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterOption`.
pub struct EaterOption {
    eater: Eater,
}

impl EaterOption {
    /// Creates a new `EaterOption`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self { eater: Eater::new(s) }
    }

    /// Analyzes the `!option` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!option")?;
        self.eater.skip_spaces();
        let key = self.eater.eat_and_get_varname()?;
        self.eater.skip_spaces();
        let value = match self.eater.eat_expression(context, memory) {
            Ok(v) => Some(v),
            Err(_) => None,
        };
        self.eater.skip_spaces();
        let option_key = OptionKey::lazy_from(&key);
        match option_key {
            None => {
                context.get_preprocessing_artifact_mut().add_warning(Warning(format!("No such !option {key}")));
            }
            Some(ref ok) => {
                if value.is_none() && ok.get_default_value().is_none() {
                    context.get_preprocessing_artifact_mut().add_warning(Warning(format!("No default value for {key}")));
                } else if value.is_none() {
                    if let Some(default) = ok.get_default_value() {
                        context.get_preprocessing_artifact_mut().get_option_mut().define(ok.clone(), default);
                    }
                } else if let Some(v) = &value {
                    context.get_preprocessing_artifact_mut().get_option_mut().define(ok.clone(), &v.to_string());
                }
            }
        }
        Ok(())
    }
}
