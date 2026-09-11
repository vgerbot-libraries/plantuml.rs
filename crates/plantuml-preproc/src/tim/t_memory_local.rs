//! Local variable memory implementation.
//!
//! Ported from `net.sourceforge.plantuml.tim.TMemoryLocal`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::execution_context_foreach::ExecutionContextForeach;
use super::execution_context_if::ExecutionContextIf;
use super::execution_context_while::ExecutionContextWhile;
use super::execution_contexts::ExecutionContexts;
use super::expression::TValue;
use super::t_memory::TMemory;
use super::t_memory_global::TMemoryGlobal;
use super::t_variable_scope::TVariableScope;
use super::trie::Trie;
use super::trie_impl::TrieImpl;
use crate::stubs::Log;
use crate::StringLocated;

/// Local variable storage — wraps global memory with local and overridden variables.
///
/// Ported from `net.sourceforge.plantuml.tim.TMemoryLocal`.
pub struct TMemoryLocal {
    contexts: ExecutionContexts,
    memory_global: TMemoryGlobal,
    overriden_variables_00: Option<TrieImpl>,
    overriden_variables_01: HashMap<String, TValue>,
    local_variables_00: TrieImpl,
    local_variables_01: HashMap<String, TValue>,
}

impl TMemoryLocal {
    /// Creates a new `TMemoryLocal` from a global memory and input variables.
    ///
    /// Ported from `TMemoryLocal` constructor.
    #[must_use]
    pub fn new(global: TMemoryGlobal, input: HashMap<String, TValue>) -> Self {
        Self {
            contexts: ExecutionContexts::default(),
            memory_global: global,
            overriden_variables_00: None,
            overriden_variables_01: input,
            local_variables_00: TrieImpl::new(),
            local_variables_01: HashMap::new(),
        }
    }
}

impl TMemory for TMemoryLocal {
    fn get_variable(&self, varname: &str) -> Option<TValue> {
        if let Some(result) = self.overriden_variables_01.get(varname) {
            return Some(result.clone());
        }
        if let Some(result) = self.memory_global.get_variable(varname) {
            return Some(result);
        }
        self.local_variables_01.get(varname).cloned()
    }

    fn put_variable(
        &mut self,
        varname: &str,
        value: TValue,
        scope: Option<TVariableScope>,
        location: &StringLocated,
    ) -> Result<(), EaterException> {
        if scope == Some(TVariableScope::Global) {
            return self.memory_global.put_variable(varname, value, scope, location);
        }
        if scope == Some(TVariableScope::Local) || self.overriden_variables_01.contains_key(varname) {
            self.overriden_variables_01.insert(varname.to_string(), value);
            if let Some(ref mut trie) = self.overriden_variables_00 {
                trie.add(varname);
            }
            Log::info(|| format!("[MemLocal/overridden] Setting {varname}"));
        } else if self.memory_global.get_variable(varname).is_some() {
            self.memory_global.put_variable(varname, value, scope, location)?;
        } else {
            self.local_variables_01.insert(varname.to_string(), value);
            self.local_variables_00.add(varname);
            Log::info(|| format!("[MemLocal/local] Setting {varname}"));
        }
        Ok(())
    }

    fn remove_variable(&mut self, varname: &str) {
        if self.overriden_variables_01.contains_key(varname) {
            self.overriden_variables_01.remove(varname);
            if let Some(ref mut trie) = self.overriden_variables_00 {
                trie.remove(varname);
            }
        } else if self.memory_global.get_variable(varname).is_some() {
            self.memory_global.remove_variable(varname);
        } else {
            self.local_variables_01.remove(varname);
            self.local_variables_00.remove(varname);
        }
    }

    fn is_empty(&self) -> bool {
        self.memory_global.is_empty() && self.local_variables_01.is_empty() && self.overriden_variables_01.is_empty()
    }

    fn variables_names(&self) -> HashSet<String> {
        // Java throws UnsupportedOperationException here
        HashSet::new()
    }

    fn variables_names3(&self) -> Box<dyn Trie> {
        let mut overriden_trie = TrieImpl::new();
        for name in self.overriden_variables_01.keys() {
            overriden_trie.add(name);
        }
        Box::new(CompositeTrie {
            global: self.memory_global.variables_names3(),
            overriden: overriden_trie,
            local: self.local_variables_00.clone(),
        })
    }

    fn fork_from_global(&self, input: HashMap<String, TValue>) -> Box<dyn TMemory> {
        Box::new(Self::new(self.memory_global.clone(), input))
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
        Log::info(|| format!("[MemLocal] Start of memory_dump {message}"));
        Log::info(|| {
            format!("[MemLocal] Number of overridden variable(s) : {}", self.overriden_variables_01.len())
        });
        let sorted: std::collections::BTreeMap<&String, &TValue> = self.overriden_variables_01.iter().collect();
        for (name, value) in &sorted {
            Log::info(|| format!("[MemLocal] {name} = {value}"));
        }
        Log::info(|| {
            format!("[MemLocal] Number of local variable(s) : {}", self.local_variables_01.len())
        });
        let sorted2: std::collections::BTreeMap<&String, &TValue> = self.local_variables_01.iter().collect();
        for (name, value) in &sorted2 {
            Log::info(|| format!("[MemLocal] {name} = {value}"));
        }
        Log::info(|| "[MemLocal] End of memory_dump".to_string());
    }
}

/// A composite trie that merges results from global, overridden, and local tries.
struct CompositeTrie {
    global: Box<dyn Trie>,
    overriden: TrieImpl,
    local: TrieImpl,
}

impl Trie for CompositeTrie {
    fn add(&mut self, _s: &str) {
        // Not supported for composite trie
    }

    fn get_longuest_match_starting_in(&self, s: &str, pos: usize) -> String {
        let s1 = self.global.get_longuest_match_starting_in(s, pos);
        let s2 = self.overriden.get_longuest_match_starting_in(s, pos);
        let s3 = self.local.get_longuest_match_starting_in(s, pos);
        if s1.len() >= s2.len() && s1.len() >= s3.len() {
            s1
        } else if s2.len() >= s3.len() && s2.len() >= s1.len() {
            s2
        } else {
            s3
        }
    }
}
