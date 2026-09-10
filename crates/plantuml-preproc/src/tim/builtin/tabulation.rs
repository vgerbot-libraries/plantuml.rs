//! Ported from `net.sourceforge.plantuml.tim.builtin.Tabulation`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;
use super::jaws;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%tab", 0));

/// `%tab()` — returns the real tabulation sentinel character.
pub struct Tabulation;

impl SimpleReturnFunction for Tabulation {}

crate::impl_simple_return_function!(
    Tabulation,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&jaws::BLOCK_E1_REAL_TABULATION.to_string()))
    },
);
