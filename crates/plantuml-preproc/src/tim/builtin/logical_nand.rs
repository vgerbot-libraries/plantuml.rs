//! Ported from `net.sourceforge.plantuml.tim.builtin.LogicalNand`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%nand", 2));

/// `%nand(...)` — logical NAND of all arguments.
pub struct LogicalNand;

impl SimpleReturnFunction for LogicalNand {}

crate::impl_simple_return_function!(
    LogicalNand,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg >= 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        for v in values {
            if !v.to_boolean() {
                return Ok(TValue::from_boolean(true));
            }
        }
        Ok(TValue::from_boolean(false))
    },
);
