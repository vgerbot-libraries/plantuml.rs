//! Execution context for `!foreach` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.ExecutionContextForeach`.

use super::iterator::code_position::CodePosition;

/// Tracks the state of a `!foreach` loop.
///
/// Ported from `net.sourceforge.plantuml.tim.ExecutionContextForeach`.
#[derive(Debug, Clone)]
pub struct ExecutionContextForeach {
    varname: String,
    json_value: serde_json::Value,
    code_position: Box<dyn CodePosition>,
    skip_me: bool,
    current_index: usize,
}

impl ExecutionContextForeach {
    /// Creates a new `ExecutionContextForeach`.
    ///
    /// Ported from `ExecutionContextForeach.fromValue`.
    #[must_use]
    pub fn from_value(
        varname: impl Into<String>,
        json_value: serde_json::Value,
        code_position: Box<dyn CodePosition>,
    ) -> Self {
        Self {
            varname: varname.into(),
            json_value,
            code_position,
            skip_me: false,
            current_index: 0,
        }
    }

    /// Marks this foreach loop to be skipped.
    pub fn skip_me_now(&mut self) {
        self.skip_me = true;
    }

    /// Returns `true` if this foreach loop should be skipped.
    #[must_use]
    pub fn is_skip_me(&self) -> bool {
        self.skip_me
    }

    /// Returns the code position where the foreach loop starts.
    #[must_use]
    pub fn get_start_foreach(&self) -> &dyn CodePosition {
        self.code_position.as_ref()
    }

    /// Returns the current value in the iteration.
    ///
    /// Ported from `ExecutionContextForeach.currentValue`.
    #[must_use]
    pub fn current_value(&self) -> serde_json::Value {
        match &self.json_value {
            serde_json::Value::Array(arr) => {
                arr.get(self.current_index)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            }
            serde_json::Value::Object(obj) => {
                if let Some((name, _)) = obj.iter().nth(self.current_index) {
                    serde_json::Value::String(name.clone())
                } else {
                    serde_json::Value::Null
                }
            }
            _ => serde_json::Value::Null,
        }
    }

    /// Increments the loop index.
    pub fn inc(&mut self) {
        self.current_index += 1;
        if self.current_index >= Self::size(&self.json_value) {
            self.skip_me = true;
        }
    }

    /// Returns the size of the JSON value (array length or object key count).
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
    pub fn get_varname(&self) -> &str {
        &self.varname
    }

    /// Returns the JSON value being iterated.
    #[must_use]
    pub fn get_json_value(&self) -> &serde_json::Value {
        &self.json_value
    }
}
