//! Ported from `net.sourceforge.plantuml.tim.builtin.StringFunction`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%string", 1));

/// `%string(v)` — converts a value to its string representation.
pub struct StringFunction;

impl SimpleReturnFunction for StringFunction {}

crate::impl_simple_return_function!(
    StringFunction,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        Ok(TValue::from_string(values[0].to_string()))
    },
);
