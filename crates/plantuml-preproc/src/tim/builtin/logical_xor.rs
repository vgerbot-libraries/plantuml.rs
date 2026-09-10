//! Ported from `net.sourceforge.plantuml.tim.builtin.LogicalXor`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%xor", 2));

/// `%xor(...)` — logical XOR (exactly one true) of all arguments.
pub struct LogicalXor;

impl SimpleReturnFunction for LogicalXor {}

crate::impl_simple_return_function!(
    LogicalXor,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg >= 2,
    execute = |_self, _context, _memory, _location, values, _named| {
        let cpt = values.iter().filter(|v| v.to_boolean()).count();
        Ok(TValue::from_boolean(cpt == 1))
    },
);
