//! Ported from `net.sourceforge.plantuml.tim.builtin.ReverseHsluvColor`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%reverse_hsluv_color", 1));

/// `%reverse_hsluv_color(color)` — reverses a color in `HSLuv` space.
///
/// TODO: implement with `HColor`/`HColorSet` once the klimt color crate is ported.
pub struct ReverseHsluvColor;

impl SimpleReturnFunction for ReverseHsluvColor {}

crate::impl_simple_return_function!(
    ReverseHsluvColor,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, _values, _named| {
        // TODO: implement with HColorSet.instance().getColor(colorString).reverseHsluv()
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
