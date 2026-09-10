//! Ported from `net.sourceforge.plantuml.tim.builtin.JsonSet`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%json_set", 3));

/// `%json_set(json, ...)` — sets a value in a JSON array or object.
pub struct JsonSet;

impl SimpleReturnFunction for JsonSet {}

crate::impl_simple_return_function!(
    JsonSet,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2 || nb_arg == 3,
    execute = |_self, _context, _memory, location, values, _named| {
        let data = &values[0];
        if !data.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let mut json = data.to_json().clone();
        match values.len() {
            2 => {
                if let serde_json::Value::Object(map) = &mut json {
                    let value = values[1].to_json();
                    if let serde_json::Value::Object(other) = &value {
                        for (k, v) in other {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                    return Ok(TValue::from_json(json));
                }
                Ok(data.clone())
            }
            3 => {
                match &mut json {
                    serde_json::Value::Array(arr) => {
                        if values[1].is_number() {
                            let index = values[1].to_int() as usize;
                            let value = values[2].to_json();
                            if index < arr.len() {
                                arr[index] = value;
                            }
                        }
                        Ok(TValue::from_json(json))
                    }
                    serde_json::Value::Object(map) => {
                        let name = values[1].to_string();
                        let value = values[2].to_json();
                        map.insert(name, value);
                        Ok(TValue::from_json(json))
                    }
                    _ => Ok(data.clone()),
                }
            }
            _ => Err(EaterException::new(
                "Error on json_set: Too many arguments".to_string(),
                location,
            )),
        }
    },
);
