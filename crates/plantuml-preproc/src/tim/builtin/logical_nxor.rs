//! Ported from `net.sourceforge.plantuml.tim.builtin.LogicalNxor`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%nxor", 2));

/// `%nxor(...)` — logical NXOR (not exactly one true) of all arguments.
pub struct LogicalNxor;

impl SimpleReturnFunction for LogicalNxor {}

crate::impl_simple_return_function!(
    LogicalNxor,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg >= 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        let cpt = values.iter().filter(|v| v.to_boolean()).count();
        Ok(TValue::from_boolean(cpt != 1))
    },
);
