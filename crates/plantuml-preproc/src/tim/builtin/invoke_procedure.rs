//! Ported from `net.sourceforge.plantuml.tim.builtin.InvokeProcedure`.
//!
//! Unlike most builtins, `InvokeProcedure` implements `TFunction` directly
//! (not `SimpleReturnFunction`) because it is a procedure, not a return function.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_context::TContext;
use crate::tim::t_function::TFunction;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::tim::t_function_type::TFunctionType;
use crate::tim::t_memory::TMemory;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%invoke_procedure", 1));

/// `%invoke_procedure(fname, ...args)` — invokes a user-defined procedure.
///
/// TODO: implement once `TContext::get_function_smart` is available and the
/// borrow checker issue is resolved.
pub struct InvokeProcedure;

impl TFunction for InvokeProcedure {
    fn get_signature(&self) -> &TFunctionSignature {
        &SIGNATURE
    }

    fn can_cover(&self, nb_arg: i32, _named_arguments: &HashSet<String>) -> bool {
        nb_arg > 0
    }

    fn get_function_type(&self) -> TFunctionType {
        TFunctionType::Procedure
    }

    fn execute_return_function(
        &self,
        _context: &mut TContext,
        _memory: &mut dyn TMemory,
        _location: &StringLocated,
        _values: &[TValue],
        _named: &HashMap<String, TValue>,
    ) -> Result<TValue, EaterException> {
        panic!("InvokeProcedure is a procedure, not a return function")
    }

    fn execute_procedure_internal(
        &self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<(), EaterException> {
        let fname = args[0].to_string();
        let sublist = &args[1..];
        let signature = TFunctionSignature::new(&fname, sublist.len() as i32);
        context.execute_procedure_by_signature(memory, location, &signature, sublist, named)
    }

    fn is_unquoted(&self) -> bool {
        false
    }
}
