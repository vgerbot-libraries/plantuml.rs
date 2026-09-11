//! Ported from `net.sourceforge.plantuml.tim.builtin.RightAlign`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;
use super::jaws;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%right_align", 0));

/// `%right_align()` — returns the right-align newline sentinel.
pub struct RightAlign;

impl SimpleReturnFunction for RightAlign {}

crate::impl_simple_return_function!(
    RightAlign,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(jaws::BLOCK_E1_NEWLINE_RIGHT_ALIGN.to_string()))
    },
);
