//! Ported from `net.sourceforge.plantuml.tim.builtin.GetAllTheme`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_all_theme", 0));

/// `%get_all_theme()` — returns a JSON array of all theme names.
///
/// TODO: implement with `ThemeUtils.getAllThemeNames()` once the theme module
/// is ported.
pub struct GetAllTheme;

impl SimpleReturnFunction for GetAllTheme {}

crate::impl_simple_return_function!(
    GetAllTheme,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        // TODO: implement with ThemeUtils.getAllThemeNames()
        Ok(TValue::from_json(serde_json::Value::Array(vec![])))
    },
);
