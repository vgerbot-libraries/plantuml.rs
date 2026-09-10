//! Ported from `net.sourceforge.plantuml.tim.builtin.Dollar`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%dollar", 0));

/// `%dollar()` — returns the literal `$` character.
pub struct Dollar;

impl SimpleReturnFunction for Dollar {}

crate::impl_simple_return_function!(
    Dollar,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string("$"))
    },
);
