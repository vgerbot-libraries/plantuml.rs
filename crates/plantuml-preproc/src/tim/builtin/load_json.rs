//! Ported from `net.sourceforge.plantuml.tim.builtin.LoadJson`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%load_json", 3));

const VALUE_CHARSET_DEFAULT: &str = "UTF-8";
const VALUE_DEFAULT_DEFAULT: &str = "{}";

/// `%load_json(path[, default[, charset]])` — loads JSON data from a file or
/// URL source.
///
/// TODO: implement with file system / URL access once the security and I/O
/// modules are ported. Currently returns the default JSON.
pub struct LoadJson;

impl SimpleReturnFunction for LoadJson {}

crate::impl_simple_return_function!(
    LoadJson,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1 || nb_arg == 2 || nb_arg == 3,
    execute = |_self, _context, _memory, location, values, _named| {
        let path = values[0].to_string();
        // TODO: implement with file system / URL access.
        // For now, try to load from the local file system.
        let default_json = if values.len() > 1 {
            values[1].to_string()
        } else {
            VALUE_DEFAULT_DEFAULT.to_string()
        };
        let _charset = if values.len() == 3 {
            values[2].to_string()
        } else {
            VALUE_CHARSET_DEFAULT.to_string()
        };

        // Try reading from file
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str(&content) {
                Ok(json) => return Ok(TValue::from_json(json)),
                Err(e) => {
                    return Err(EaterException::new(
                        format!("JSON parse issue in source {path} on location {e}"),
                        location,
                    ));
                }
            }
        }
        // Fall back to default
        serde_json::from_str(&default_json).map_or_else(|_| Ok(TValue::from_string("")), |json| Ok(TValue::from_json(json)))
    },
);
