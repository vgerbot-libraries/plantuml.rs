//! TMemory trait — variable storage interface.
//!
//! Ported from `net.sourceforge.plantuml.tim.TMemory`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::execution_context_foreach::ExecutionContextForeach;
use super::execution_context_if::ExecutionContextIf;
use super::execution_context_while::ExecutionContextWhile;
use super::expression::TValue;
use super::trie::Trie;
use super::t_variable_scope::TVariableScope;
use crate::StringLocated;

/// Trait for variable storage in the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.tim.TMemory`.
pub trait TMemory {
    /// Gets a variable by name.
    fn get_variable(&self, varname: &str) -> Option<TValue>;

    /// Puts a variable with the given scope.
    fn put_variable(
        &mut self,
        varname: &str,
        value: TValue,
        scope: Option<TVariableScope>,
        location: &StringLocated,
    ) -> Result<(), EaterException>;

    /// Removes a variable.
    fn remove_variable(&mut self, varname: &str);

    /// Returns `true` if the memory is empty.
    fn is_empty(&self) -> bool;

    /// Returns the names of all variables.
    fn variables_names(&self) -> HashSet<String>;

    /// Returns a trie of variable names for fast prefix matching.
    fn variables_names3(&self) -> Box<dyn Trie>;

    /// Forks a new memory from the global memory with the given input.
    fn fork_from_global(&self, input: HashMap<String, TValue>) -> Box<dyn TMemory>;

    /// Peeks at the top if context.
    fn peek_if(&self) -> Option<&ExecutionContextIf>;

    /// Returns `true` if all if conditions are satisfied.
    fn are_all_if_ok(&self) -> bool;

    /// Adds an if context.
    fn add_if(&mut self, context: ExecutionContextIf);

    /// Adds a while context.
    fn add_while(&mut self, value: ExecutionContextWhile);

    /// Adds a foreach context.
    fn add_foreach(&mut self, value: ExecutionContextForeach);

    /// Pops the top if context.
    fn poll_if(&mut self) -> Option<ExecutionContextIf>;

    /// Pops the top while context.
    fn poll_while(&mut self) -> Option<ExecutionContextWhile>;

    /// Peeks at the top while context.
    fn peek_while(&self) -> Option<&ExecutionContextWhile>;

    /// Pops the top foreach context.
    fn poll_foreach(&mut self) -> Option<ExecutionContextForeach>;

    /// Peeks at the top foreach context.
    fn peek_foreach(&self) -> Option<&ExecutionContextForeach>;

    /// Dumps debug info about the memory.
    fn dump_debug(&self, message: &str);
}
