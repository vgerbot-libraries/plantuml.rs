//! Ported from `net.sourceforge.plantuml.tim.builtin.Strlen`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%strlen", 1));

/// `%strlen(s)` — returns the length of a string.
pub struct Strlen;

impl SimpleReturnFunction for Strlen {}

crate::impl_simple_return_function!(
    Strlen,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        Ok(TValue::from_int(values[0].to_string().chars().count() as i32))
    },
);
