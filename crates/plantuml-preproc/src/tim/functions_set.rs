//! Functions set — manages all registered preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.FunctionsSet`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::eater_legacy_define::EaterLegacyDefine;
use super::eater_legacy_define_long::EaterLegacyDefineLong;
use super::eater_declare_procedure::EaterDeclareProcedure;
use super::eater_declare_return_function::EaterDeclareReturnFunction;
use super::t_context::TContext;
use super::t_function::TFunction;
use super::t_function_impl::TFunctionImpl;
use super::t_function_signature::TFunctionSignature;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use super::trie::Trie;
use super::trie_impl::TrieImpl;
use crate::StringLocated;

/// Manages all registered preprocessor functions (builtins, user-defined, legacy defines).
///
/// Ported from `net.sourceforge.plantuml.tim.FunctionsSet`.
pub struct FunctionsSet {
    functions: HashMap<TFunctionSignature, Box<dyn TFunction>>,
    functions_by_name: HashMap<String, HashMap<TFunctionSignature, Box<dyn TFunction>>>,
    functions_final: HashSet<TFunctionSignature>,
    functions3: TrieImpl,
    pending_function: Option<TFunctionImpl>,
}

impl FunctionsSet {
    /// Creates a new empty `FunctionsSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            functions_by_name: HashMap::new(),
            functions_final: HashSet::new(),
            functions3: TrieImpl::new(),
            pending_function: None,
        }
    }

    /// Returns the number of registered functions.
    #[must_use]
    pub fn size(&self) -> usize {
        self.functions.len()
    }

    /// Returns the longest function name match starting at position `pos` in `s`.
    ///
    /// Ported from `FunctionsSet.getLonguestMatchStartingIn`.
    pub fn get_longuest_match_starting_in(&self, s: &str, pos: usize) -> String {
        self.functions3.get_longuest_match_starting_in(s, pos)
    }

    /// Returns the pending function (being defined), if any.
    #[must_use]
    pub fn pending_function(&self) -> Option<&TFunctionImpl> {
        self.pending_function.as_ref()
    }

    /// Returns the pending function (mutable), if any.
    pub fn pending_function_mut(&mut self) -> Option<&mut TFunctionImpl> {
        self.pending_function.as_mut()
    }

    /// Adds a function to the set.
    pub fn add_function(&mut self, func: Box<dyn TFunction>) {
        let name = func.get_signature().get_function_name().to_string();
        let key = func.get_signature().clone();
        self.functions3.add(&format!("{name}("));
        self.update_functions_by_name(&name, key, func);
    }

    fn update_functions_by_name(
        &mut self,
        name: &str,
        key: TFunctionSignature,
        func: Box<dyn TFunction>,
    ) {
        self.functions_by_name
            .entry(name.to_string())
            .or_default()
            .insert(key, func);
    }

    /// Returns `true` if at least one function with the given name exists.
    ///
    /// Ported from `FunctionsSet.doesFunctionExist`.
    #[must_use]
    pub fn does_function_exist(&self, function_name: &str) -> bool {
        self.functions_by_name.contains_key(function_name)
    }

    /// Returns the functions matching the given name.
    ///
    /// Ported from `FunctionsSet.getFunctionsByName`.
    pub fn get_functions_by_name(&self, function_name: &str) -> Vec<&dyn TFunction> {
        self.functions_by_name.get(function_name).map_or_else(Vec::new, |map| map.values().map(std::convert::AsRef::as_ref).collect())
    }

    /// Finds a function matching the given signature, using smart matching.
    ///
    /// Ported from `FunctionsSet.getFunctionSmart`.
    pub fn get_function_smart(&self, searched: &TFunctionSignature) -> Option<&dyn TFunction> {
        if let Some(map) = self.functions_by_name.get(searched.get_function_name()) {
            // Try exact match first
            for (sig, func) in map {
                if sig == searched {
                    return Some(func.as_ref());
                }
            }
            // Try canCover match
            for (sig, func) in map {
                if sig.same_function_name_as(searched) && func.can_cover(searched.get_nb_arg(), searched.get_named_arguments()) {
                    return Some(func.as_ref());
                }
            }
        }
        None
    }

    /// Finalizes the pending function (called when `!endfunction`/`!endprocedure` is encountered).
    pub fn execute_endfunction(&mut self) {
        if let Some(mut pending) = self.pending_function.take() {
            if pending.get_function_type() == TFunctionType::LegacyDefineLong {
                pending.finalize_enddefinelong();
            }
            self.add_function(Box::new(pending));
        }
    }

    /// Executes a legacy `!define` directive.
    pub fn execute_legacy_define(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Result<(), EaterException> {
        if self.pending_function.is_some() {
            return Err(EaterException::new("already0048", s));
        }
        let mut legacy_define = EaterLegacyDefine::new(s.clone());
        legacy_define.analyze(context, memory)?;
        if let Some(func) = legacy_define.take_function() {
            let name = func.get_signature().get_function_name().to_string();
            let key = func.get_signature().clone();
            self.functions3.add(&format!("{name}("));
            self.update_functions_by_name(&name, key, Box::new(func));
        }
        Ok(())
    }

    /// Executes a `!definelong` directive.
    pub fn execute_legacy_define_long(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Result<(), EaterException> {
        if self.pending_function.is_some() {
            return Err(EaterException::new("already0068", s));
        }
        let mut legacy_define_long = EaterLegacyDefineLong::new(s.clone());
        legacy_define_long.analyze(context, memory)?;
        self.pending_function = legacy_define_long.take_function();
        Ok(())
    }

    /// Executes a `!function` declaration.
    pub fn execute_declare_return_function(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Result<(), EaterException> {
        if self.pending_function.is_some() {
            return Err(EaterException::new("already0068", s));
        }
        let mut declare_function = EaterDeclareReturnFunction::new(s.clone());
        declare_function.analyze(context, memory)?;
        let final_flag = declare_function.get_final_flag();
        if let Some(func) = declare_function.take_function() {
            let declared_signature = func.get_signature().clone();
            let previous_exists = self.functions_by_name
                .get(declared_signature.get_function_name())
                .is_some_and(|m| m.contains_key(&declared_signature));
            if previous_exists && (final_flag || self.functions_final.contains(&declared_signature)) {
                return Err(EaterException::new("This function is already defined", s));
            }
            if final_flag {
                self.functions_final.insert(declared_signature);
            }
            if func.has_body() {
                self.add_function(Box::new(func));
            } else {
                self.pending_function = Some(func);
            }
        }
        Ok(())
    }

    /// Executes a `!procedure` declaration.
    pub fn execute_declare_procedure(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Result<(), EaterException> {
        if self.pending_function.is_some() {
            return Err(EaterException::new("already0068", s));
        }
        let mut declare_function = EaterDeclareProcedure::new(s.clone());
        declare_function.analyze(context, memory)?;
        let final_flag = declare_function.get_final_flag();
        if let Some(func) = declare_function.take_function() {
            let declared_signature = func.get_signature().clone();
            let previous_exists = self.functions_by_name
                .get(declared_signature.get_function_name())
                .is_some_and(|m| m.contains_key(&declared_signature));
            if previous_exists && (final_flag || self.functions_final.contains(&declared_signature)) {
                return Err(EaterException::new("This function is already defined", s));
            }
            if final_flag {
                self.functions_final.insert(declared_signature);
            }
            if func.has_body() {
                self.add_function(Box::new(func));
            } else {
                self.pending_function = Some(func);
            }
        }
        Ok(())
    }
}

impl Default for FunctionsSet {
    fn default() -> Self {
        Self::new()
    }
}
