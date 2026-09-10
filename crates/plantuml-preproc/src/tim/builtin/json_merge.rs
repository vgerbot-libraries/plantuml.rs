//! Ported from `net.sourceforge.plantuml.tim.builtin.JsonMerge`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%json_merge", 2));

/// `%json_merge(json1, json2)` — merges two JSON values (arrays or objects).
pub struct JsonMerge;

impl SimpleReturnFunction for JsonMerge {}

crate::impl_simple_return_function!(
    JsonMerge,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let data0 = &values[0];
        if !data0.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let data1 = &values[1];
        if !data1.is_json() {
            return Err(EaterException::new("Not JSON data".to_string(), location));
        }
        let json0 = data0.to_json().clone();
        let json1 = data1.to_json();
        match (json0, json1) {
            (serde_json::Value::Array(mut a0), serde_json::Value::Array(a1)) => {
                a0.extend(a1);
                Ok(TValue::from_json(serde_json::Value::Array(a0)))
            }
            (serde_json::Value::Object(mut m0), serde_json::Value::Object(m1)) => {
                for (k, v) in m1 {
                    m0.insert(k, v);
                }
                Ok(TValue::from_json(serde_json::Value::Object(m0)))
            }
            _ => Ok(data0.clone()),
        }
    },
);
