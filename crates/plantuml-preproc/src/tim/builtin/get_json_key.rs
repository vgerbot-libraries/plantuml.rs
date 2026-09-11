//! Ported from `net.sourceforge.plantuml.tim.builtin.GetJsonKey`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_json_keys", 1));

/// `%get_json_keys(json)` — returns a JSON array of keys from a JSON object,
/// or all keys from objects in a JSON array.
pub struct GetJsonKey;

impl SimpleReturnFunction for GetJsonKey {}

crate::impl_simple_return_function!(
    GetJsonKey,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, values, _named| {
        let data = &values[0];
        if !data.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let json = data.to_json();
        let mut keys: Vec<serde_json::Value> = vec![];
        if let serde_json::Value::Object(map) = json {
            for key in map.keys() {
                keys.push(serde_json::Value::String(key.clone()));
            }
            return Ok(TValue::from_json(serde_json::Value::Array(keys)));
        }
        if let serde_json::Value::Array(arr) = json {
            for item in arr {
                if let serde_json::Value::Object(map) = item {
                    for key in map.keys() {
                        keys.push(serde_json::Value::String(key.clone()));
                    }
                }
            }
            return Ok(TValue::from_json(serde_json::Value::Array(keys)));
        }
        Err(EaterException::new("Bad JSON type".to_string(), location))
    },
);
