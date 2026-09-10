//! Ported from `net.sourceforge.plantuml.tim.builtin.Dec2hex`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%dec2hex", 1));

/// `%dec2hex(n)` — converts a decimal integer to a hex string.
pub struct Dec2hex;

impl SimpleReturnFunction for Dec2hex {}

crate::impl_simple_return_function!(
    Dec2hex,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let n = values[0].to_int();
        Ok(TValue::from_string(&format!("{n:x}")))
    },
);
