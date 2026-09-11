//! Ported from `net.sourceforge.plantuml.tim.builtin.Chr`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%chr", 1));

/// `%chr(code)` — converts a code point to a character.
pub struct Chr;

impl SimpleReturnFunction for Chr {}

crate::impl_simple_return_function!(
    Chr,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let code = values[0].to_int();
        char::from_u32(code as u32).map_or_else(|| Ok(TValue::from_string("\0")), |c| Ok(TValue::from_string(c.to_string())))
    },
);
