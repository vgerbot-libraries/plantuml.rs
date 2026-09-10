//! Ported from `net.sourceforge.plantuml.tim.builtin.FilenameNoExtension`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::preproc::defines::Defines;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%filename_no_extension", 0));

/// `%filename_no_extension()` — returns the filename without extension from
/// the Defines environment.
pub struct FilenameNoExtension {
    value: String,
}

impl FilenameNoExtension {
    /// Creates a new `FilenameNoExtension` from the given `Defines`.
    pub fn new(defines: &Defines) -> Self {
        Self {
            value: defines
                .get_environment_value("filenameNoExtension")
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl SimpleReturnFunction for FilenameNoExtension {}

crate::impl_simple_return_function!(
    FilenameNoExtension,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |self_, _context, _memory, _location, _values, _named| {
        Ok(TValue::from_string(&self_.value))
    },
);
