//! Ported from `net.sourceforge.plantuml.tim.builtin.FunctionExists`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%function_exists", 1));

/// `%function_exists(name)` — returns true if a user-defined function exists.
pub struct FunctionExists;

impl SimpleReturnFunction for FunctionExists {}

crate::impl_simple_return_function!(
    FunctionExists,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, context, _memory, _location, values, _named| {
        let name = values[0].to_string();
        Ok(TValue::from_boolean(context.does_function_exist(&name)))
    },
);
