//! Ported from `net.sourceforge.plantuml.tim.builtin.SplitStr`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%splitstr", 3));

/// `%splitstr(str, separator)` — splits a string by any of the separator
/// characters and returns a JSON array of tokens.
pub struct SplitStr;

impl SimpleReturnFunction for SplitStr {}

crate::impl_simple_return_function!(
    SplitStr,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        let str_val = values[0].to_string();
        let separators = values[1].to_string();
        let sep_chars: Vec<char> = separators.chars().collect();
        let mut tokens: Vec<serde_json::Value> = vec![];
        let mut current = String::new();
        for c in str_val.chars() {
            if sep_chars.contains(&c) {
                tokens.push(serde_json::Value::String(std::mem::take(&mut current)));
            } else {
                current.push(c);
            }
        }
        tokens.push(serde_json::Value::String(current));
        Ok(TValue::from_json(serde_json::Value::Array(tokens)))
    },
);
