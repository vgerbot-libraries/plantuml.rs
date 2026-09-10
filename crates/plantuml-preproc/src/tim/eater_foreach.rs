//! Eater for `!foreach` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterForeach`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses `!foreach` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterForeach`.
pub struct EaterForeach {
    eater: Eater,
    varname: Option<String>,
    json_value: Option<serde_json::Value>,
}

impl EaterForeach {
    /// Creates a new `EaterForeach`.
    #[must_use]
    pub fn new(s: StringLocated) -> Self {
        Self {
            eater: Eater::new(s),
            varname: None,
            json_value: None,
        }
    }

    /// Analyzes the `!foreach` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!foreach")?;
        self.eater.skip_spaces();
        self.varname = Some(self.eater.eat_and_get_varname()?);
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("in")?;
        self.eater.skip_spaces();
        let value = self.eater.eat_expression(context, memory)?;
        self.json_value = Some(value.to_json());
        Ok(())
    }

    /// Returns `true` if the foreach should be skipped (empty collection).
    #[must_use]
    pub fn is_skip(&self) -> bool {
        match &self.json_value {
            Some(json) => Self::size(json) == 0,
            None => true,
        }
    }

    /// Returns the size of a JSON value.
    ///
    /// Ported from `EaterForeach.size`.
    #[must_use]
    pub fn size(value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Array(arr) => arr.len(),
            serde_json::Value::Object(obj) => obj.len(),
            _ => 0,
        }
    }

    /// Returns the variable name.
    #[must_use]
    pub fn get_varname(&self) -> Option<&str> {
        self.varname.as_deref()
    }

    /// Returns the JSON value.
    #[must_use]
    pub fn get_json_value(&self) -> Option<&serde_json::Value> {
        self.json_value.as_ref()
    }
}
