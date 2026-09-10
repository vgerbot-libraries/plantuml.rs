//! Ported from `net.sourceforge.plantuml.tim.builtin.Backslash`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;
use super::jaws;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%backslash", 0));

/// `%backslash()` — returns the real backslash sentinel character.
pub struct Backslash;

impl SimpleReturnFunction for Backslash {}

crate::impl_simple_return_function!(
    Backslash,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&jaws::BLOCK_E1_REAL_BACKSLASH.to_string()))
    },
);
