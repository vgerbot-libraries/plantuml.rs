//! Ported from `net.sourceforge.plantuml.tim.builtin.Filedate`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::preproc::defines::Defines;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%filedate", 0));

/// `%filedate()` — returns the file date from the Defines environment.
pub struct Filedate {
    value: String,
}

impl Filedate {
    /// Creates a new `Filedate` from the given `Defines`.
    pub fn new(defines: &Defines) -> Self {
        Self {
            value: defines.get_environment_value("filedate").unwrap_or_default().to_string(),
        }
    }
}

impl SimpleReturnFunction for Filedate {}

crate::impl_simple_return_function!(
    Filedate,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |self_, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&self_.value))
    },
);
