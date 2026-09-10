//! Ported from `net.sourceforge.plantuml.tim.builtin.BoolVal`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%boolval", 1));

/// `%boolval(s)` — converts a string to a boolean.
pub struct BoolVal;

impl SimpleReturnFunction for BoolVal {}

crate::impl_simple_return_function!(
    BoolVal,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, values, _named| {
        let s = values[0].to_string().to_lowercase();
        if s == "true" || s == "1" {
            Ok(TValue::from_boolean(true))
        } else if s == "false" || s == "0" {
            Ok(TValue::from_boolean(false))
        } else {
            Err(EaterException::new(
                format!("Cannot convert {s} to boolean."),
                location,
            ))
        }
    },
);
