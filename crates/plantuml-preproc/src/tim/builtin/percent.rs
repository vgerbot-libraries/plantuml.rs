//! Ported from `net.sourceforge.plantuml.tim.builtin.Percent`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%percent", 0));

/// `%percent()` — returns the literal `%` character.
pub struct Percent;

impl SimpleReturnFunction for Percent {}

crate::impl_simple_return_function!(
    Percent,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string("%"))
    },
);
