//! Ported from `net.sourceforge.plantuml.tim.builtin.Filename`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::preproc::defines::Defines;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%filename", 0));

/// `%filename()` — returns the filename from the Defines environment.
pub struct Filename {
    value: String,
}

impl Filename {
    /// Creates a new `Filename` from the given `Defines`.
    pub fn new(defines: &Defines) -> Self {
        Self {
            value: defines.get_environment_value("filename").unwrap_or_default().to_string(),
        }
    }
}

impl SimpleReturnFunction for Filename {}

crate::impl_simple_return_function!(
    Filename,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |self_, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&self_.value))
    },
);
