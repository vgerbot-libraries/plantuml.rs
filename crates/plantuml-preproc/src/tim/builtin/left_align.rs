//! Ported from `net.sourceforge.plantuml.tim.builtin.LeftAlign`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;
use super::jaws;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%left_align", 0));

/// `%left_align()` — returns the left-align newline sentinel.
pub struct LeftAlign;

impl SimpleReturnFunction for LeftAlign {}

crate::impl_simple_return_function!(
    LeftAlign,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(jaws::BLOCK_E1_NEWLINE_LEFT_ALIGN.to_string()))
    },
);
