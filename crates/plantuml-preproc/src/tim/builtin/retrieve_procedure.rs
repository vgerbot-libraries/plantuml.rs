//! Ported from `net.sourceforge.plantuml.tim.builtin.RetrieveProcedure`.

use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%retrieve_procedure", 1));

/// `%retrieve_procedure(fname, ...args)` — invokes a procedure and returns
/// the text it output.
///
/// TODO: implement once `TContext::get_function_smart`,
/// `TContext::get_result_list`, and `TContext::extract_from_result_list`
/// are available and the borrow checker issue is resolved.
pub struct RetrieveProcedure;

impl SimpleReturnFunction for RetrieveProcedure {}

crate::impl_simple_return_function!(
    RetrieveProcedure,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg > 0,
    execute = |_self, context, memory, location, values, _named| {
        let fname = values[0].to_string();
        let args = &values[1..];
        let signature = TFunctionSignature::new(&fname, args.len() as i32);
        let n1 = context.get_result_list().len();
        context.execute_procedure_by_signature(
            memory,
            location,
            &signature,
            args,
            &std::collections::HashMap::new(),
        )?;
        let extracted = context.extract_from_result_list(n1);
        Ok(TValue::from_string(extracted))
    },
);
