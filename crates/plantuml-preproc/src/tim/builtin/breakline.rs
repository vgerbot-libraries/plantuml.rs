//! Ported from `net.sourceforge.plantuml.tim.builtin.Breakline`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;
use super::{jaws, jaws_flags};

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%breakline", 0));

/// `%breakline()` — returns a breakline sentinel or `\n`.
pub struct Breakline;

impl SimpleReturnFunction for Breakline {}

crate::impl_simple_return_function!(
    Breakline,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        if jaws_flags::USE_BLOCK_E1_IN_NEWLINE_FUNCTION {
            Ok(TValue::from_string(&jaws::BLOCK_E1_BREAKLINE.to_string()))
        } else {
            Ok(TValue::from_string("\n"))
        }
    },
);
