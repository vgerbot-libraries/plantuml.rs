//! Ported from `net.sourceforge.plantuml.tim.builtin.Strpos`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%strpos", 2));

/// `%strpos(haystack, needle)` — returns the index of `needle` in `haystack`, or -1.
pub struct Strpos;

impl SimpleReturnFunction for Strpos {}

crate::impl_simple_return_function!(
    Strpos,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        let full = values[0].to_string();
        let searched = values[1].to_string();
        // Java's String.indexOf returns -1 when not found, and uses UTF-16 code unit index.
        // Rust's str::find returns byte index. We use char-based search to match Java semantics.
        let result = full
            .find(&searched[..])
            .map(|byte_idx| full[..byte_idx].chars().count() as i32)
            .unwrap_or(-1);
        Ok(TValue::from_int(result))
    },
);
