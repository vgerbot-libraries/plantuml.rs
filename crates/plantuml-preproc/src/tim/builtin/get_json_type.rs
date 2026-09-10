//! Ported from `net.sourceforge.plantuml.tim.builtin.GetJsonType`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_json_type", 1));

/// `%get_json_type(value)` — returns the type name of a JSON value.
pub struct GetJsonType;

impl SimpleReturnFunction for GetJsonType {}

crate::impl_simple_return_function!(
    GetJsonType,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let data = &values[0];
        if data.is_string() {
            return Ok(TValue::from_string("string"));
        }
        if data.is_number() {
            return Ok(TValue::from_string("number"));
        }
        if !data.is_json() {
            return Ok(TValue::from_string("not_json"));
        }
        let json = data.to_json();
        match json {
            serde_json::Value::Array(_) => Ok(TValue::from_string("array")),
            serde_json::Value::Object(_) => Ok(TValue::from_string("object")),
            serde_json::Value::Bool(_) => Ok(TValue::from_string("boolean")),
            serde_json::Value::Number(_) => Ok(TValue::from_string("number")),
            serde_json::Value::String(_) => Ok(TValue::from_string("string")),
            serde_json::Value::Null => Ok(TValue::from_string("json")),
        }
    },
);
