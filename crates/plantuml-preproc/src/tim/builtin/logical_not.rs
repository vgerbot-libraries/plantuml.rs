//! Ported from `net.sourceforge.plantuml.tim.builtin.LogicalNot`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%not", 1));

/// `%not(b)` — logical NOT.
pub struct LogicalNot;

impl SimpleReturnFunction for LogicalNot {}

crate::impl_simple_return_function!(
    LogicalNot,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        Ok(TValue::from_boolean(!values[0].to_boolean()))
    },
);
