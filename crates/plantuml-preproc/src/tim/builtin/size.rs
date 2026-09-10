//! Ported from `net.sourceforge.plantuml.tim.builtin.Size`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%size", 1));

/// `%size(value)` — returns the size of a string, JSON array, or JSON object.
pub struct Size;

impl SimpleReturnFunction for Size {}

crate::impl_simple_return_function!(
    Size,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let value = &values[0];
        if value.is_number() {
            return Ok(TValue::from_int(0));
        }
        if value.is_string() {
            return Ok(TValue::from_int(value.to_string().chars().count() as i32));
        }
        let json = value.to_json();
        match json {
            serde_json::Value::Array(arr) => Ok(TValue::from_int(arr.len() as i32)),
            serde_json::Value::Object(map) => Ok(TValue::from_int(map.len() as i32)),
            _ => Ok(TValue::from_int(0)),
        }
    },
);
