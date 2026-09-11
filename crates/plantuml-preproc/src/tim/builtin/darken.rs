//! Ported from `net.sourceforge.plantuml.tim.builtin.Darken`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%darken", 2));

/// `%darken(color, ratio)` — darkens a color by the given ratio.
///
/// TODO: implement with `HColor`/`HColorSet` once the klimt color crate is ported.
pub struct Darken;

impl SimpleReturnFunction for Darken {}

crate::impl_simple_return_function!(
    Darken,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let _color_string = values[0].to_string();
        // TODO: implement with HColorSet.instance().getColor(colorString).darken(ratio)
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
