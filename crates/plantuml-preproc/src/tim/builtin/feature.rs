//! Ported from `net.sourceforge.plantuml.tim.builtin.Feature`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%feature", 1));

/// `%feature(name)` — returns 1 if the feature is supported, 0 otherwise.
pub struct Feature;

impl SimpleReturnFunction for Feature {}

crate::impl_simple_return_function!(
    Feature,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let arg = values[0].to_string();
        if arg.eq_ignore_ascii_case("style") || arg.eq_ignore_ascii_case("theme") {
            Ok(TValue::from_int(1))
        } else {
            Ok(TValue::from_int(0))
        }
    },
);
