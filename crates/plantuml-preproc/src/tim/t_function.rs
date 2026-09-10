//! TFunction trait — interface for preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.TFunction`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_function_signature::TFunctionSignature;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Trait for preprocessor functions (builtins, user-defined, legacy defines).
///
/// Ported from `net.sourceforge.plantuml.tim.TFunction`.
pub trait TFunction: Send + Sync {
    /// Returns the function signature.
    fn get_signature(&self) -> &TFunctionSignature;

    /// Returns `true` if this function can cover the given arg count and named arguments.
    fn can_cover(&self, nb_arg: i32, named_arguments: &HashSet<String>) -> bool;

    /// Returns the function type.
    fn get_function_type(&self) -> TFunctionType;

    /// Executes this function as a return function.
    fn execute_return_function(
        &self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<TValue, EaterException>;

    /// Executes this function as a procedure.
    fn execute_procedure_internal(
        &self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<(), EaterException>;

    /// Returns `true` if this function is unquoted.
    fn is_unquoted(&self) -> bool;
}
