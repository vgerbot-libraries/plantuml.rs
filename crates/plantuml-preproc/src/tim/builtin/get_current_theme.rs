//! Ported from `net.sourceforge.plantuml.tim.builtin.GetCurrentTheme`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%get_current_theme", 0));

/// `%get_current_theme()` — returns the metadata of the current theme.
///
/// TODO: implement with `TContext::get_theme_metadata()` once available.
pub struct GetCurrentTheme;

impl SimpleReturnFunction for GetCurrentTheme {}

crate::impl_simple_return_function!(
    GetCurrentTheme,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0,
    execute = |_self, _context, _memory, _location, _values, _named| {
        // TODO: implement with context.get_theme_metadata() once available.
        Ok(TValue::from_json(serde_json::Value::Object(
            serde_json::Map::new(),
        )))
    },
);
