//! Ported from `net.sourceforge.plantuml.tim.builtin.AlwaysFalse`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%false", 0));

/// `%false()` — always returns `false`.
pub struct AlwaysFalse;

impl SimpleReturnFunction for AlwaysFalse {}

crate::impl_simple_return_function!(
    AlwaysFalse,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_boolean(false))
    },
);
