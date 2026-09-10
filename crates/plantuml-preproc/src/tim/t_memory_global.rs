//! Global variable memory implementation.
//!
//! Ported from `net.sourceforge.plantuml.tim.TMemoryGlobal`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::execution_context_foreach::ExecutionContextForeach;
use super::execution_context_if::ExecutionContextIf;
use super::execution_context_while::ExecutionContextWhile;
use super::execution_contexts::ExecutionContexts;
use super::expression::TValue;
use super::t_memory::TMemory;
use super::t_variable_scope::TVariableScope;
use super::trie::Trie;
use super::trie_impl::TrieImpl;
use crate::stubs::Log;
use crate::StringLocated;

/// Global variable storage for the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.tim.TMemoryGlobal`.
#[derive(Debug, Clone, Default)]
pub struct TMemoryGlobal {
    contexts: ExecutionContexts,
    global_variables: HashMap<String, TValue>,
    variables: TrieImpl,
}

impl TMemoryGlobal {
    /// Creates a new `TMemoryGlobal`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn dump_memory_internal(&self) {
        Log::info(|| format!("[MemGlobal] Number of variable(s) : {}", self.global_variables.len()));
        let mut sorted: std::collections::BTreeMap<&String, &TValue> = self.global_variables.iter().collect();
        for (name, value) in &sorted {
            Log::info(|| format!("[MemGlobal] {name} = {value}"));
        }
    }
}

impl TMemory for TMemoryGlobal {
    fn get_variable(&self, varname: &str) -> Option<TValue> {
        self.global_variables.get(varname).cloned()
    }

    fn put_variable(
        &mut self,
        varname: &str,
        value: TValue,
        scope: Option<TVariableScope>,
        location: &StringLocated,
    ) -> Result<(), EaterException> {
        if scope == Some(TVariableScope::Local) {
            return Err(EaterException::new("Cannot use local variable here", &location));
        }
        Log::info(|| format!("[MemGlobal] Setting {varname}"));
        self.global_variables.insert(varname.to_string(), value);
        self.variables.add(varname);
        Ok(())
    }

    fn remove_variable(&mut self, varname: &str) {
        self.global_variables.remove(varname);
        self.variables.remove(varname);
    }

    fn is_empty(&self) -> bool {
        self.global_variables.is_empty()
    }

    fn variables_names(&self) -> HashSet<String> {
        self.global_variables.keys().cloned().collect()
    }

    fn variables_names3(&self) -> Box<dyn Trie> {
        Box::new(self.variables.clone())
    }

    fn fork_from_global(&self, input: HashMap<String, TValue>) -> Box<dyn TMemory> {
        Box::new(super::t_memory_local::TMemoryLocal::new(self.clone(), input))
    }

    fn peek_if(&self) -> Option<&ExecutionContextIf> {
        self.contexts.peek_if()
    }

    fn are_all_if_ok(&self) -> bool {
        self.contexts.are_all_if_ok()
    }

    fn add_if(&mut self, context: ExecutionContextIf) {
        self.contexts.add_if(context);
    }

    fn add_while(&mut self, value: ExecutionContextWhile) {
        self.contexts.add_while(value);
    }

    fn add_foreach(&mut self, value: ExecutionContextForeach) {
        self.contexts.add_foreach(value);
    }

    fn poll_if(&mut self) -> Option<ExecutionContextIf> {
        self.contexts.poll_if()
    }

    fn poll_while(&mut self) -> Option<ExecutionContextWhile> {
        self.contexts.poll_while()
    }

    fn peek_while(&self) -> Option<&ExecutionContextWhile> {
        self.contexts.peek_while()
    }

    fn poll_foreach(&mut self) -> Option<ExecutionContextForeach> {
        self.contexts.poll_foreach()
    }

    fn peek_foreach(&self) -> Option<&ExecutionContextForeach> {
        self.contexts.peek_foreach()
    }

    fn dump_debug(&self, message: &str) {
        Log::info(|| format!("[MemGlobal] Start of memory_dump {message}"));
        self.dump_memory_internal();
        Log::info(|| "[MemGlobal] End of memory_dump".to_string());
    }
}
