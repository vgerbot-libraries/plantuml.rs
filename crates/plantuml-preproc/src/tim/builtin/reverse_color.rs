//! Ported from `net.sourceforge.plantuml.tim.builtin.ReverseColor`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%reverse_color", 1));

/// `%reverse_color(color)` — reverses a color.
///
/// TODO: implement with `HColor`/`HColorSet` once the klimt color crate is ported.
pub struct ReverseColor;

impl SimpleReturnFunction for ReverseColor {}

crate::impl_simple_return_function!(
    ReverseColor,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, location, _values, _named| {
        // TODO: implement with HColorSet.instance().getColor(colorString).reverse()
        Err(EaterException::new(
            "No such color".to_string(),
            location,
        ))
    },
);
