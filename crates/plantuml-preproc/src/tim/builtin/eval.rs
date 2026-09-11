//! Ported from `net.sourceforge.plantuml.tim.builtin.Eval`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%eval", 1));

/// `%eval(expr)` — evaluates a numeric expression.
///
/// In Java this delegates to `StringEater.eatExpression`. The Rust port
/// requires the `StringEater` type from `PortTimCore`.
pub struct Eval;

impl SimpleReturnFunction for Eval {}

crate::impl_simple_return_function!(
    Eval,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        // TODO: implement with StringEater.eatExpression once PortTimCore
        // provides the StringEater type. For now, try a simple integer parse.
        let exp = values[0].to_string();
        exp.trim().parse::<i32>().map_or_else(|_| Ok(TValue::from_int(0)), |n| Ok(TValue::from_int(n)))
    },
);
