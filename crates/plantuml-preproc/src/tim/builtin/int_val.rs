//! Ported from `net.sourceforge.plantuml.tim.builtin.IntVal`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%intval", 1));

/// `%intval(s)` — converts a string to an integer.
pub struct IntVal;

impl SimpleReturnFunction for IntVal {}

crate::impl_simple_return_function!(
    IntVal,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, values, _named| {
        let s = values[0].to_string();
        s.parse::<i32>().map_or_else(|_| Err(EaterException::new(
                format!("Cannot convert {s} to integer."),
                location,
            )), |n| Ok(TValue::from_int(n)))
    },
);
