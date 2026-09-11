//! Ported from `net.sourceforge.plantuml.tim.builtin.Getenv`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%getenv", 1));

/// `%getenv(name)` — returns the value of an environment variable.
///
/// In Java this checks `SecurityUtils.getSecurityProfile()` before reading.
/// The Rust port uses `std::env::var` directly.
pub struct Getenv;

impl SimpleReturnFunction for Getenv {}

crate::impl_simple_return_function!(
    Getenv,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let name = values[0].to_string();
        // TODO: implement SecurityUtils check for server deployments.
        std::env::var(&name).map_or_else(|_| Ok(TValue::from_string("")), |v| Ok(TValue::from_string(&v)))
    },
);
