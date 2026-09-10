//! Ported from `net.sourceforge.plantuml.tim.builtin.SetVariableValue`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::tim::t_variable_scope::TVariableScope;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%set_variable_value", 2));

/// `%set_variable_value(name, value)` — sets a global variable and returns
/// an empty string.
pub struct SetVariableValue;

impl SimpleReturnFunction for SetVariableValue {}

crate::impl_simple_return_function!(
    SetVariableValue,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, memory, location, values, _named| {
        let name = values[0].to_string();
        let value = values[1].clone();
        memory.put_variable(&name, value, Some(TVariableScope::Global), location)?;
        Ok(TValue::from_string(""))
    },
);
