//! Ported from `net.sourceforge.plantuml.tim.builtin.HslColor`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%hsl_color", 3));

/// `%hsl_color(h, s, l[, a])` — creates a color from HSL components.
///
/// TODO: implement with `HSLColor` once the klimt color crate is ported.
pub struct HslColor;

impl SimpleReturnFunction for HslColor {}

crate::impl_simple_return_function!(
    HslColor,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 3 || nb_arg == 4,
    execute = |_self, _context, _memory, location, values, _named| {
        let _h = values[0].to_int();
        let _s = values[1].to_int();
        let _l = values[2].to_int();
        // TODO: implement with new HSLColor(h, s, l).getRGB()
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
