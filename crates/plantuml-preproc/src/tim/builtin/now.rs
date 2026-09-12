//! Ported from `net.sourceforge.plantuml.tim.builtin.Now`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%now", 0));

/// `%now()` — returns the current Unix timestamp in seconds.
pub struct Now;

impl SimpleReturnFunction for Now {}

crate::impl_simple_return_function!(
    Now,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        let now = crate::wasm_time::now_secs() as i32;
        Ok(TValue::from_int(now))
    },
);
