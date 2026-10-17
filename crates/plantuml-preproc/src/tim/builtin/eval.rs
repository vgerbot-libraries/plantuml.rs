//! Ported from `net.sourceforge.plantuml.tim.builtin.Eval`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::string_eater::StringEater;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%eval", 1));

/// `%eval(expr)` — evaluates an expression and returns its integer value.
///
/// Ported from `net.sourceforge.plantuml.tim.builtin.Eval`: the argument is
/// fed through [`StringEater`]'s expression pipeline, matching
/// `new StringEater(exp).eatExpression(context, memory)`, and the result is
/// converted with `TValue.fromInt(value.toInt())`.
pub struct Eval;

impl SimpleReturnFunction for Eval {}

crate::impl_simple_return_function!(
    Eval,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, context, memory, _location, values, _named| {
        let exp = values[0].to_string();
        let mut eater = StringEater::new(exp);
        let value = eater.get_eater_mut().eat_expression(context, memory)?;
        Ok(TValue::from_int(value.to_int()))
    },
);
