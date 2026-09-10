//! Ported from `net.sourceforge.plantuml.tim.builtin.GetVariableValue`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_variable_value", 1));

/// `%get_variable_value(name)` — returns the variable value or empty string.
pub struct GetVariableValue;

impl SimpleReturnFunction for GetVariableValue {}

crate::impl_simple_return_function!(
    GetVariableValue,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, memory, _location, values, _named| {
        let name = values[0].to_string();
        match memory.get_variable(&name) {
            Some(v) => Ok(v),
            None => Ok(TValue::from_string("")),
        }
    },
);
