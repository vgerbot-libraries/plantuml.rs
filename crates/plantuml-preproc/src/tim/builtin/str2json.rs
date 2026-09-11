//! Ported from `net.sourceforge.plantuml.tim.builtin.Str2Json`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%str2json", 1));

/// `%str2json(str)` — parses a JSON string into a JSON value.
pub struct Str2Json;

impl SimpleReturnFunction for Str2Json {}

crate::impl_simple_return_function!(
    Str2Json,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let value = values[0].to_string();
        serde_json::from_str(&value).map_or_else(|_| Ok(TValue::from_string("")), |json| Ok(TValue::from_json(json)))
    },
);
