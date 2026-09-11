//! Ported from `net.sourceforge.plantuml.tim.builtin.Ord`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%ord", 1));

/// `%ord(s)` — returns the Unicode code point of the first character.
pub struct Ord;

impl SimpleReturnFunction for Ord {}

crate::impl_simple_return_function!(
    Ord,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let s = values[0].to_string();
        s.chars().next().map_or_else(|| Ok(TValue::from_int(0)), |c| Ok(TValue::from_int(c as i32)))
    },
);
