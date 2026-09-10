//! Ported from `net.sourceforge.plantuml.tim.builtin.JsonKeyExists`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%json_key_exists", 1));

/// `%json_key_exists(json, key)` — returns true if the JSON object contains
/// the given key.
pub struct JsonKeyExists;

impl SimpleReturnFunction for JsonKeyExists {}

crate::impl_simple_return_function!(
    JsonKeyExists,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        let arg0 = &values[0];
        if !arg0.is_json() {
            return Ok(TValue::from_boolean(false));
        }
        let json = arg0.to_json();
        let serde_json::Value::Object(map) = json else {
            return Ok(TValue::from_boolean(false));
        };
        let arg1 = &values[1];
        if arg1.is_string() || (arg1.is_json() && matches!(arg1.to_json(), serde_json::Value::String(_))) {
            let keyname = arg1.to_string();
            return Ok(TValue::from_boolean(map.contains_key(&keyname)));
        }
        Ok(TValue::from_boolean(false))
    },
);
