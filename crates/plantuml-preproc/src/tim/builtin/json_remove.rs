//! Ported from `net.sourceforge.plantuml.tim.builtin.JsonRemove`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%json_remove", 2));

/// `%json_remove(json, key_or_index)` — removes an element from a JSON array
/// or a key from a JSON object.
pub struct JsonRemove;

impl SimpleReturnFunction for JsonRemove {}

crate::impl_simple_return_function!(
    JsonRemove,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let data = &values[0];
        if !data.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let mut json = data.to_json().clone();
        match &mut json {
            serde_json::Value::Array(arr) => {
                if values[1].is_number() {
                    let index = values[1].to_int() as usize;
                    if index < arr.len() {
                        arr.remove(index);
                    }
                }
                Ok(TValue::from_json(json))
            }
            serde_json::Value::Object(map) => {
                let name = values[1].to_string();
                map.remove(&name);
                Ok(TValue::from_json(json))
            }
            _ => Ok(data.clone()),
        }
    },
);
