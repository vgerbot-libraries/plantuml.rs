//! Ported from `net.sourceforge.plantuml.tim.builtin.IsLight`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%is_light", 1));

/// `%is_light(color)` — returns true if the color is light.
///
/// TODO: implement with `HColor`/`HColorSet` once the klimt color crate is ported.
pub struct IsLight;

impl SimpleReturnFunction for IsLight {}

crate::impl_simple_return_function!(
    IsLight,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, _values, _named| {
        // TODO: implement with HColorSet.instance().getColor(colorString).isDark()
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
