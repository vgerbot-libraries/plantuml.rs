//! Ported from `net.sourceforge.plantuml.tim.builtin.GetVersion`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%version", 0));

/// `%version()` — returns the `PlantUML` version string.
pub struct GetVersion;

impl SimpleReturnFunction for GetVersion {}

crate::impl_simple_return_function!(
    GetVersion,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        // TODO: implement with Version.versionString() once available.
        Ok(TValue::from_string("1.2024.8"))
    },
);
