//! Ported from `net.sourceforge.plantuml.tim.builtin.GetStdlib`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_stdlib", 1));

/// `%get_stdlib()` or `%get_stdlib(folder)` or `%get_stdlib(folder, key)` —
/// returns stdlib metadata as JSON, or a specific value.
///
/// TODO: implement with `Stdlib` once the stdlib module is ported.
pub struct GetStdlib;

impl SimpleReturnFunction for GetStdlib {}

crate::impl_simple_return_function!(
    GetStdlib,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0 || nb_arg == 1 || nb_arg == 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        // TODO: implement with Stdlib.getAllFolderNames() and Stdlib.retrieve()
        match values.len() {
            2 => Ok(TValue::from_string("")),
            _ => Ok(TValue::from_json(serde_json::Value::Object(
                serde_json::Map::new(),
            ))),
        }
    },
);
