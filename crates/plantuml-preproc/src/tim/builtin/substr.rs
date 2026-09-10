//! Ported from `net.sourceforge.plantuml.tim.builtin.Substr`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%substr", 3));

/// `%substr(s, pos[, len])` — extracts a substring.
pub struct Substr;

impl SimpleReturnFunction for Substr {}

crate::impl_simple_return_function!(
    Substr,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2 || nb_arg == 3,
    execute = |_self, _context, _memory, _location, values, _named| {
        let full = values[0].to_string();
        let pos = values[1].to_int();
        let chars: Vec<char> = full.chars().collect();
        if pos as usize >= chars.len() {
            return Ok(TValue::from_string(""));
        }
        let remaining: String = chars[pos as usize..].iter().collect();
        if values.len() == 3 {
            let len = values[2].to_int() as usize;
            if len < remaining.chars().count() {
                return Ok(TValue::from_string(&remaining[..remaining.char_indices().nth(len).map(|(i, _)| i).unwrap_or(remaining.len())]));
            }
        }
        Ok(TValue::from_string(&remaining))
    },
);
