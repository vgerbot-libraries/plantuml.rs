//! Ported from `net.sourceforge.plantuml.tim.builtin.LogicalOr`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%or", 2));

/// `%or(...)` — logical OR of all arguments.
pub struct LogicalOr;

impl SimpleReturnFunction for LogicalOr {}

crate::impl_simple_return_function!(
    LogicalOr,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg >= 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        for v in values {
            if v.to_boolean() {
                return Ok(TValue::from_boolean(true));
            }
        }
        Ok(TValue::from_boolean(false))
    },
);
