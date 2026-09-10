//! Ported from `net.sourceforge.plantuml.tim.builtin.Upper`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%upper", 1));

/// `%upper(s)` — converts a string to uppercase.
pub struct Upper;

impl SimpleReturnFunction for Upper {}

crate::impl_simple_return_function!(
    Upper,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        Ok(TValue::from_string(&values[0].to_string().to_uppercase()))
    },
);
