//! Ported from `net.sourceforge.plantuml.tim.builtin.SimpleReturnFunction`.
//!
//! In Java, `SimpleReturnFunction` is an abstract class that provides default
//! implementations of `TFunction` for simple return-type builtins:
//! - `getFunctionType()` → `RETURN_FUNCTION`
//! - `executeProcedureInternal()` → throws `UnsupportedOperationException`
//! - `isUnquoted()` → `false`
//!
//! In Rust, we use a trait to mark simple return functions and a macro to
//! generate the common `TFunction` boilerplate.

use std::collections::{HashMap, HashSet};

use crate::string_located::StringLocated;
use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_context::TContext;
use crate::tim::t_function::TFunction;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::tim::t_function_type::TFunctionType;
use crate::tim::t_memory::TMemory;

/// Marker trait for simple return-type builtin functions.
///
/// Ported from `net.sourceforge.plantuml.tim.builtin.SimpleReturnFunction`.
///
/// Types implementing this trait should use the [`impl_simple_return_function!`]
/// macro to generate the `TFunction` implementation with the standard defaults
/// for return-type functions.
pub trait SimpleReturnFunction: TFunction {}

/// Macro to implement the `TFunction` trait for a simple return function builtin.
///
/// This provides the default implementations for `get_function_type`,
/// `execute_procedure_internal`, and `is_unquoted`, matching the Java
/// `SimpleReturnFunction` abstract class.
///
/// # Example
/// ```ignore
/// impl_simple_return_function!(
///     Chr,
///     &*SIGNATURE,
///     |nb_arg, _named| nb_arg == 1,
///     |_self, _context, _memory, _location, values, _named| {
///         let code = values[0].to_int();
///         let s = char::from_u32(code as u32).map(|c| c.to_string()).unwrap_or_default();
///         Ok(TValue::from_string(&s))
///     }
/// );
/// ```
#[macro_export]
macro_rules! impl_simple_return_function {
    (
        $type:ty,
        $sig:expr,
        can_cover = |$nb_arg:ident, $named1:ident| $can_cover_body:expr,
        execute = |$self:ident, $ctx:ident, $mem:ident, $loc:ident, $vals:ident, $named2:ident| $execute_body:expr $(,)?
    ) => {
        impl $crate::tim::t_function::TFunction for $type {
            fn get_signature(&self) -> &$crate::tim::t_function_signature::TFunctionSignature {
                $sig
            }
            fn can_cover(
                &self,
                $nb_arg: i32,
                $named1: &std::collections::HashSet<String>,
            ) -> bool {
                $can_cover_body
            }
            fn get_function_type(&self) -> $crate::tim::t_function_type::TFunctionType {
                $crate::tim::t_function_type::TFunctionType::ReturnFunction
            }
            fn execute_return_function(
                &self,
                $ctx: &mut $crate::tim::t_context::TContext,
                $mem: &mut dyn $crate::tim::t_memory::TMemory,
                $loc: &$crate::string_located::StringLocated,
                $vals: &[$crate::tim::expression::TValue],
                $named2: &std::collections::HashMap<
                    String,
                    $crate::tim::expression::TValue,
                >,
            ) -> Result<
                $crate::tim::expression::TValue,
                $crate::tim::eater_exception::EaterException,
            > {
                let $self = self;
                $execute_body
            }
            fn execute_procedure_internal(
                &self,
                _context: &mut $crate::tim::t_context::TContext,
                _memory: &mut dyn $crate::tim::t_memory::TMemory,
                _location: &$crate::string_located::StringLocated,
                _args: &[$crate::tim::expression::TValue],
                _named: &std::collections::HashMap<
                    String,
                    $crate::tim::expression::TValue,
                >,
            ) -> Result<(), $crate::tim::eater_exception::EaterException> {
                panic!(
                    "UnsupportedOperationException: SimpleReturnFunction does not support execute_procedure_internal"
                )
            }
            fn is_unquoted(&self) -> bool {
                false
            }
        }
    };
}
