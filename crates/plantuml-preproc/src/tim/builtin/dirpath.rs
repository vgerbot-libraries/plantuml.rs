//! Ported from `net.sourceforge.plantuml.tim.builtin.Dirpath`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::preproc::defines::Defines;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%dirpath", 0));

/// `%dirpath()` — returns the directory path from the Defines environment.
pub struct Dirpath {
    value: String,
}

impl Dirpath {
    /// Creates a new `Dirpath` from the given `Defines`.
    pub fn new(defines: &Defines) -> Self {
        Self {
            value: defines.get_environment_value("dirpath").unwrap_or_default().to_string(),
        }
    }
}

impl SimpleReturnFunction for Dirpath {}

crate::impl_simple_return_function!(
    Dirpath,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |self_, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&self_.value))
    },
);
