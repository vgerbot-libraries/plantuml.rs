//! Ported from `net.sourceforge.plantuml.tim.builtin.Modulo`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%mod", 2));

/// `%mod(a, b)` — returns `a % b`.
pub struct Modulo;

impl SimpleReturnFunction for Modulo {}

crate::impl_simple_return_function!(
    Modulo,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 2,
    execute = |_self, _context, _memory, location, values, _named| {
        let dividend = values[0].to_int();
        let divisor = values[1].to_int();
        if divisor == 0 {
            return Err(EaterException::new("Divide by zero".to_string(), location));
        }
        Ok(TValue::from_int(dividend % divisor))
    },
);
