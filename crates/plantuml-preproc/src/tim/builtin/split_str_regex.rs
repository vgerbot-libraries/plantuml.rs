//! Ported from `net.sourceforge.plantuml.tim.builtin.SplitStrRegex`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%splitstr_regex", 2));

/// `%splitstr_regex(str, regex)` — splits a string by a regex pattern and
/// returns a JSON array of parts.
pub struct SplitStrRegex;

impl SimpleReturnFunction for SplitStrRegex {}

crate::impl_simple_return_function!(
    SplitStrRegex,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let str_val = values[0].to_string();
        let separator = values[1].to_string();
        let re = regex::Regex::new(&separator).map_err(|_| {
            EaterException::new(
                format!("Invalid regex: {separator}"),
                location,
            )
        })?;
        let parts: Vec<serde_json::Value> = re
            .split(&str_val)
            .map(|s| serde_json::Value::String(s.to_string()))
            .collect();
        Ok(TValue::from_json(serde_json::Value::Array(parts)))
    },
);
