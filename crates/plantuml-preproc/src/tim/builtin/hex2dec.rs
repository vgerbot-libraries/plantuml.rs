//! Ported from `net.sourceforge.plantuml.tim.builtin.Hex2dec`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%hex2dec", 1));

/// `%hex2dec(hex)` — converts a hex string to a decimal integer.
pub struct Hex2dec;

impl SimpleReturnFunction for Hex2dec {}

crate::impl_simple_return_function!(
    Hex2dec,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let s = values[0].to_string();
        match i32::from_str_radix(&s, 16) {
            Ok(n) => Ok(TValue::from_int(n)),
            Err(_) => Ok(TValue::from_int(0)),
        }
    },
);
