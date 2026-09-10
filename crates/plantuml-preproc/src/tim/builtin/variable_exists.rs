//! Ported from `net.sourceforge.plantuml.tim.builtin.VariableExists`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%variable_exists", 1));

/// `%variable_exists(name)` — returns true if the variable is defined.
pub struct VariableExists;

impl SimpleReturnFunction for VariableExists {}

crate::impl_simple_return_function!(
    VariableExists,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, memory, _location, values, _named| {
        let name = values[0].to_string();
        Ok(TValue::from_boolean(memory.get_variable(&name).is_some()))
    },
);
