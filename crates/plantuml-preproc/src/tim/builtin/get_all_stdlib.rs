//! Ported from `net.sourceforge.plantuml.tim.builtin.GetAllStdlib`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_all_stdlib", 1));

/// `%get_all_stdlib()` or `%get_all_stdlib(detail)` — returns a JSON array of
/// stdlib folder names, or a JSON object with details.
///
/// TODO: implement with `Stdlib` once the stdlib module is ported.
pub struct GetAllStdlib;

impl SimpleReturnFunction for GetAllStdlib {}

crate::impl_simple_return_function!(
    GetAllStdlib,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0 || nb_arg == 1,
    execute = |_self, _context, _memory, location, values, _named| {
        // TODO: implement with Stdlib.getAllFolderNames()
        match values.len() {
            0 => Ok(TValue::from_json(serde_json::Value::Array(vec![]))),
            1 => Ok(TValue::from_json(serde_json::Value::Object(
                serde_json::Map::new(),
            ))),
            _ => Err(EaterException::new(
                "Error on get_all_stdlib: Too many arguments".to_string(),
                location,
            )),
        }
    },
);
