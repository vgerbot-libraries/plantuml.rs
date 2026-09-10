//! Ported from `net.sourceforge.plantuml.tim.builtin.Lighten`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%lighten", 2));

/// `%lighten(color, ratio)` — lightens a color by the given ratio.
///
/// TODO: implement with `HColor`/`HColorSet` once the klimt color crate is ported.
pub struct Lighten;

impl SimpleReturnFunction for Lighten {}

crate::impl_simple_return_function!(
    Lighten,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let _color_string = values[0].to_string();
        // TODO: implement with HColorSet.instance().getColor(colorString).lighten(ratio)
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
