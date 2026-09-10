//! Ported from `net.sourceforge.plantuml.tim.builtin.CallUserFunction`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%call_user_func", 1));

/// `%call_user_func(fname, ...args)` — calls a user-defined return function.
///
/// TODO: implement once `TContext::get_function_smart` is available and the
/// borrow checker issue (immutable borrow of context for lookup followed by
/// mutable borrow for execution) is resolved.
pub struct CallUserFunction;

impl SimpleReturnFunction for CallUserFunction {}

crate::impl_simple_return_function!(
    CallUserFunction,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg > 0,
    execute = |_self, context, memory, location, values, named| {
        let fname = values[0].to_string();
        let args = &values[1..];
        let signature = TFunctionSignature::new(&fname, args.len() as i32);
        context.execute_return_function_by_signature(memory, location, &signature, args, named)
    },
);
