//! Ported from `net.sourceforge.plantuml.tim.builtin.JsonAdd`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%json_add", 3));

/// `%json_add(json, value)` or `%json_add(json, key, value)` — adds an element
/// to a JSON array or a key-value pair to a JSON object.
pub struct JsonAdd;

impl SimpleReturnFunction for JsonAdd {}

crate::impl_simple_return_function!(
    JsonAdd,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2 || nb_arg == 3,
    execute = |_self, _context, _memory, location, values, _named| {
        let data = &values[0];
        if !data.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let mut json = data.to_json();
        match &mut json {
            serde_json::Value::Array(arr) => {
                let value = values[1].to_json();
                arr.push(value);
                Ok(TValue::from_json(json))
            }
            serde_json::Value::Object(map) => {
                if values.len() < 3 {
                    return Err(EaterException::new("Bad JSON type".to_string(), location));
                }
                let name = values[1].to_string();
                let value = values[2].to_json();
                map.insert(name, value);
                Ok(TValue::from_json(json))
            }
            _ => Ok(data.clone()),
        }
    },
);
