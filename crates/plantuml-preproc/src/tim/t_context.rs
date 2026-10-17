//! `TContext` — the main preprocessor orchestrator.
//!
//! Ported from `net.sourceforge.plantuml.tim.TContext`.

use std::collections::{HashMap, HashSet};

use super::eater_affectation::EaterAffectation;
use super::eater_affectation_define::EaterAffectationDefine;
use super::eater_assert::EaterAssert;
use super::eater_exception::EaterException;
use super::eater_foreach::EaterForeach;
use super::eater_if::EaterIf;
use super::eater_ifdef::EaterIfdef;
use super::eater_ifndef::EaterIfndef;
use super::eater_import::EaterImport;
use super::eater_includesub::EaterIncludesub;
use super::eater_include::EaterInclude;
use super::eater_include_def::EaterIncludeDef;
use super::eater_log::EaterLog;
use super::eater_option::EaterOption;
use super::eater_return::EaterReturn;
use super::eater_startsub::EaterStartsub;
use super::eater_theme::EaterTheme;
use super::eater_undef::EaterUndef;
use super::eater_while::EaterWhile;
use super::eater_else_if::EaterElseIf;
use super::eater_dump_memory::EaterDumpMemory;
use super::execution_context_foreach::ExecutionContextForeach;
use super::execution_context_if::ExecutionContextIf;
use super::execution_context_while::ExecutionContextWhile;
use super::expression::TValue;
use super::functions_set::FunctionsSet;
use super::t_function::TFunction;
use super::t_function_signature::TFunctionSignature;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;

use crate::preproc::{PreprocessingArtifact, Sub};
use crate::stubs::{DefinitionsContainer, PathSystem};
use crate::{StringLocated, TLineType};

/// The main preprocessor orchestrator. Manages function/variable substitution,
// directive execution, and result collection.
///
/// Ported from `net.sourceforge.plantuml.tim.TContext`.
pub struct TContext {
    result_list: Vec<StringLocated>,
    debug: Vec<StringLocated>,
    functions_set: FunctionsSet,
    #[allow(dead_code)]
    charset: String,
    subs: HashMap<String, Sub>,
    #[allow(dead_code)]
    definitions_container: DefinitionsContainer,
    files_used_current: HashSet<String>,
    preprocessing_artifact: PreprocessingArtifact,
    path_system: PathSystem,
    pending_add: Option<String>,
    // Current read index in `execute_lines_internal`, used by `!while` /
    // `!foreach` to jump back to the loop start.
    reading_index: usize,
}

impl TContext {
    /// Creates a new `TContext`.
    ///
    /// Ported from `TContext` constructor.
    /// Creates a new `TContext` with the given defines.
    ///
    /// Ported from `TContext(PathSystem, Defines, Charset, DefinitionsContainer)`.
    #[must_use]
    pub fn new(defines: &crate::preproc::defines::Defines) -> Self {
        let mut context = Self {
            result_list: Vec::new(),
            debug: Vec::new(),
            functions_set: FunctionsSet::new(),
            charset: String::new(),
            pending_add: None,
            reading_index: 0,
            subs: HashMap::new(),
            definitions_container: DefinitionsContainer,
            files_used_current: HashSet::new(),
            preprocessing_artifact: PreprocessingArtifact::new(),
            path_system: PathSystem::default(),
        };
        context.add_standard_functions(defines);
        context
    }

    /// Appends the failing line, tagged with a preprocessor error.
    ///
    /// Ported from `TimLoader.load`'s catch block:
    /// `context.getResultList().add(e.getLocation().withErrorPreprocessor(msg))`.
    pub fn append_preprocessor_error(&mut self, location: &StringLocated, message: &str) {
        self.result_list
            .push(location.with_error_preprocessor(message));
    }

    /// Returns the files used in the current preprocessing.
    #[must_use]
    pub fn get_files_used_current(&self) -> &HashSet<String> {
        &self.files_used_current
    }

    /// Returns the preprocessing artifact.
    #[must_use]
    pub fn get_preprocessing_artifact(&self) -> &PreprocessingArtifact {
        &self.preprocessing_artifact
    }

    /// Returns the preprocessing artifact (mutable).
    pub fn get_preprocessing_artifact_mut(&mut self) -> &mut PreprocessingArtifact {
        &mut self.preprocessing_artifact
    }

    /// Sets the current directory for file resolution (used by `!includesub`).
    pub fn set_current_dir(&mut self, dir: impl Into<std::path::PathBuf>) {
        self.path_system.set_current_dir(dir);
    }


    /// Returns the functions set.
    #[must_use]
    pub fn functions_set(&self) -> &FunctionsSet {
        &self.functions_set
    }

    /// Returns the functions set (mutable).
    pub fn functions_set_mut(&mut self) -> &mut FunctionsSet {
        &mut self.functions_set
    }

    /// Returns the subs map.
    #[must_use]
    pub fn get_subs(&self) -> &HashMap<String, Sub> {
        &self.subs
    }

    /// Returns the result list.
    #[must_use]
    pub fn get_result_list(&self) -> &[StringLocated] {
        &self.result_list
    }

    /// Returns the debug list.
    #[must_use]
    pub fn get_debug(&self) -> &[StringLocated] {
        &self.debug
    }

    /// Returns the path system.
    #[must_use]
    pub fn get_path_system(&self) -> &PathSystem {
        &self.path_system
    }

    /// Returns `true` if a function with the given name exists.
    ///
    /// Ported from `TContext.doesFunctionExist`.
    #[must_use]
    pub fn does_function_exist(&self, function_name: &str) -> bool {
        self.functions_set.does_function_exist(function_name)
    }

    /// Returns `true` if the function is a legacy define.
    ///
    /// Ported from `TContext.isLegacyDefine`.
    pub fn is_legacy_define(&self, function_name: &str) -> bool {
        for func in self.functions_set.get_functions_by_name(function_name) {
            if func.get_function_type().is_legacy() {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the function is unquoted.
    ///
    /// Ported from `TContext.isUnquoted`.
    pub fn is_unquoted(&self, function_name: &str) -> bool {
        for func in self.functions_set.get_functions_by_name(function_name) {
            if func.is_unquoted() {
                return true;
            }
        }
        false
    }

    /// Returns the function name at the given position in a string.
    ///
    /// Ported from `TContext.getFunctionNameAt`.
    pub fn get_function_name_at(&self, s: &str, pos: usize) -> Option<String> {
        // Check if we're just after a letter (to avoid matching in the middle of words)
        let just_after_a_letter = pos > 0
            && crate::TLineType::is_letter_or_emoji_or_underscore_or_digit(
                s.as_bytes().get(pos - 1).copied().map_or('\0', |b| b as char),
            )
            && !super::variable_manager::VariableManager::just_after_backslash_n(s, pos);
        if just_after_a_letter
            && s.as_bytes().get(pos).copied().map_or('\0', |b| b as char) != '%'
            && s.as_bytes().get(pos).copied().map_or('\0', |b| b as char) != '$'
        {
            return None;
        }

        let fname = self.functions_set.get_longuest_match_starting_in(s, pos);
        if fname.is_empty() {
            return None;
        }
        // The Trie stores function names with "(" appended (e.g. "%eval("),
        // so strip the trailing "(" to get the actual function name.
        Some(fname[..fname.len() - 1].to_string())
    }
    /// Returns the function matching the given signature using smart matching.
    ///
    /// Ported from `TContext.getFunctionSmart`.
    pub fn get_function_smart(&self, searched: &TFunctionSignature) -> Option<&dyn TFunction> {
        self.functions_set.get_function_smart(searched)
    }

    /// Registers all standard (builtin) functions.
    ///
    /// Ported from `TContext.addStandardFunctions`.
    fn add_standard_functions(&mut self, defines: &crate::preproc::defines::Defines) {
        super::builtin::register_all(&mut self.functions_set, defines);
    }

    /// Executes a list of preprocessor lines.
    ///
    /// Ported from `TContext.executeLines`.
    pub fn execute_lines(
        &mut self,
        memory: &mut dyn TMemory,
        lines: &[StringLocated],
        function_type: Option<TFunctionType>,
        in_function: bool,
    ) -> Result<Option<TValue>, EaterException> {
        self.execute_lines_internal(memory, lines, function_type, in_function)
    }

    /// Executes preprocessor lines internally.
    ///
    /// Ported from `TContext.executeLinesInternal`.
    #[allow(clippy::only_used_in_recursion)]
    fn execute_lines_internal(
        &mut self,
        memory: &mut dyn TMemory,
        lines: &[StringLocated],
        function_type: Option<TFunctionType>,
        in_function: bool,
    ) -> Result<Option<TValue>, EaterException> {
        // Save outer reading state for nested procedure calls.
        let saved_index = self.reading_index;
        let mut current_sub: Option<(String, Vec<StringLocated>)> = None;
        let mut i = 0;
        while i < lines.len() {
            self.reading_index = i + 1;
            let line = lines[i].clone();
            let t = line.get_type();

            // Handle !startsub/!endsub: collect lines into sub-blocks, then
            // process them recursively (procedure definitions need registration).
            if let Some((name, sub_lines)) = current_sub.as_mut() {
                if t == TLineType::Endsub {
                    let name = name.clone();
                    let collected = std::mem::take(sub_lines);
                    // Store in subs map
                    let mut sub = crate::preproc::sub::Sub::new(&name);
                    for sl in &collected {
                        sub.add(sl.clone());
                    }
                    self.subs.insert(name, sub);
                    current_sub = None;
                    // Recursively process the sub-block lines
                    self.execute_lines_internal(memory, &collected, function_type, in_function)?;
                    i += 1;
                    continue;
                }
                sub_lines.push(line);
                i += 1;
                continue;
            }

            if t == TLineType::Startsub {
                let mut eater = EaterStartsub::new(line.clone());
                eater.analyze(self, memory)?;
                let subname = eater.get_subname().unwrap_or("").to_string();
                current_sub = Some((subname, Vec::new()));
                i += 1;
                continue;
            }
            // Collect body lines for pending !procedure/!function definitions
            if self.functions_set.pending_function().is_some() {
                if t == TLineType::EndFunction {
                    self.functions_set.execute_endfunction();
                    i += 1;
                    continue;
                }
                // Add line to pending function body, don't process as normal line
                let mut fs = std::mem::take(&mut self.functions_set);
                if let Some(pending) = fs.pending_function_mut() {
                    let _ = pending.add_body(line);
                }
                self.functions_set = fs;
                i += 1;
                continue;
            }




            match t {
                TLineType::Plain => {
                    if memory.are_all_if_ok() {
                        self.add_plain(memory, &line)?;
                    }
                }
                TLineType::Affectation => {
                    if memory.are_all_if_ok() {
                        let mut eater = EaterAffectation::new(line.clone());
                        eater.analyze(self, memory)?;
                    }
                }
                TLineType::AffectationDefine => {
                    if memory.are_all_if_ok() {
                        let mut eater = EaterAffectationDefine::new(line.clone());
                        eater.analyze(self, memory)?;
                    }
                }
                TLineType::Assert => {
                    if memory.are_all_if_ok() {
                        let mut eater = EaterAssert::new(line.clone());
                        eater.analyze(self, memory)?;
                    }
                }
                TLineType::If => {
                    let mut eater = EaterIf::new(line.clone());
                    eater.analyze(self, memory)?;
                    memory.add_if(ExecutionContextIf::from_value(eater.is_true()));
                }
                TLineType::Ifdef => {
                    let mut eater = EaterIfdef::new(line.clone());
                    eater.analyze(self, memory)?;
                    memory.add_if(ExecutionContextIf::from_value(eater.is_true(self, memory)));
                }
                TLineType::Ifndef => {
                    let mut eater = EaterIfndef::new(line.clone());
                    eater.analyze(self, memory)?;
                    memory.add_if(ExecutionContextIf::from_value(eater.is_true(self, memory)));
                }
                TLineType::Else => {
                    if let Some(ctx) = memory.poll_if() {
                        let mut ctx = ctx;
                        ctx.now_in_else();
                        memory.add_if(ctx);
                    }
                }
                TLineType::ElseIf => {
                    let mut eater = EaterElseIf::new(line.clone());
                    let prev_if = memory.poll_if();
                    if let Some(mut ctx) = prev_if {
                        ctx.entering_else_if();
                        if !ctx.has_been_burn() {
                            eater.analyze(self, memory)?;
                            if eater.is_true() {
                                ctx.now_in_some_else_if();
                            }
                        }
                        memory.add_if(ctx);
                    }
                }
                TLineType::Endif => {
                    memory.poll_if();
                }
                TLineType::While => {
                    let mut eater = EaterWhile::new(line.clone());
                    eater.analyze(self, memory)?;
                    let condition_value = if let Some(expr) = eater.take_while_expression() {
                        expr.get_result(&line, self, memory)?
                    } else {
                        TValue::from_boolean(false)
                    };
                    let code_position = i;
                    if condition_value.to_boolean() {
                        memory.add_while(ExecutionContextWhile::from_value(
                            eater.take_while_expression().unwrap_or_default(),
                            Box::new(super::iterator::Position { pos: code_position }),
                        ));
                    } else {
                        // Skip to matching !endwhile
                        let mut depth = 1;
                        while i + 1 < lines.len() {
                            i += 1;
                            let lt = lines[i].get_type();
                            if lt == TLineType::While {
                                depth += 1;
                            } else if lt == TLineType::Endwhile {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
                TLineType::Endwhile => {
                    if let Some(ctx) = memory.peek_while().cloned() {
                        if ctx.is_skip_me() {
                            memory.poll_while();
                        } else {
                            // Re-evaluate condition and jump back
                            let condition_value = ctx.condition_value(&line, self, memory)?;
                            if condition_value.to_boolean() {
                                // Jump back to start of while
                                let start = ctx.get_start_while();
                                if let Some(pos) = start.as_any().downcast_ref::<super::iterator::Position>() {
                                    self.reading_index = pos.pos;
                                }
                            } else {
                                memory.poll_while();
                            }
                        }
                    }
                }
                TLineType::Foreach => {
                    let mut eater = EaterForeach::new(line.clone());
                    eater.analyze(self, memory)?;
                    if eater.is_skip() {
                        // Skip to matching !endfor
                        let mut depth = 1;
                        while i + 1 < lines.len() {
                            i += 1;
                            let lt = lines[i].get_type();
                            if lt == TLineType::Foreach {
                                depth += 1;
                            } else if lt == TLineType::Endforeach {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                    } else {
                        let varname = eater.get_varname().unwrap_or("").to_string();
                        let json_value = eater.get_json_value().cloned().unwrap_or(serde_json::Value::Null);
                        let code_position = i;
                        memory.add_foreach(ExecutionContextForeach::from_value(
                            varname,
                            json_value,
                            Box::new(super::iterator::Position { pos: code_position }),
                        ));
                    }
                }
                TLineType::Endforeach => {
                    if let Some(mut ctx) = memory.peek_foreach().cloned() {
                        ctx.inc();
                        if ctx.is_skip_me() {
                            memory.poll_foreach();
                        } else {
                            // Set the loop variable and jump back
                            let value = ctx.current_value();
                            let varname = ctx.get_varname().to_string();
                            memory.put_variable(&varname, TValue::from_json(value), None, &line)?;
                            let start = ctx.get_start_foreach();
                            if let Some(pos) = start.as_any().downcast_ref::<super::iterator::Position>() {
                                self.reading_index = pos.pos;
                            }
                        }
                    }
                }
                TLineType::DeclareReturnFunction => {
                    let mut fs = std::mem::take(&mut self.functions_set);
                    fs.execute_declare_return_function(self, memory, &line)?;
                    self.functions_set = fs;
                }
                TLineType::DeclareProcedure => {
                    let mut fs = std::mem::take(&mut self.functions_set);
                    fs.execute_declare_procedure(self, memory, &line)?;
                    self.functions_set = fs;
                }
                // EndFunction handled before match (pending function body collection)

                TLineType::Return => {
                    if in_function {
                        let mut eater = EaterReturn::new(line.clone());
                        eater.analyze(self, memory)?;
                        return Ok(eater.take_value());
                    }
                    return Err(EaterException::new("!return outside of function", &line));
                }
                TLineType::LegacyDefine => {
                    let mut fs = std::mem::take(&mut self.functions_set);
                    fs.execute_legacy_define(self, memory, &line)?;
                    self.functions_set = fs;
                }
                TLineType::LegacyDefineLong => {
                    let mut fs = std::mem::take(&mut self.functions_set);
                    fs.execute_legacy_define_long(self, memory, &line)?;
                    self.functions_set = fs;
                }
                TLineType::Theme => {
                    self.execute_theme(memory, &line)?;
                }
                TLineType::Include => {
                    self.execute_include(memory, &line)?;
                }
                TLineType::IncludeDef => {
                    self.execute_include_def(memory, &line)?;
                }
                TLineType::Import => {
                    self.execute_import(memory, &line)?;
                }
                TLineType::Includesub => {
                    self.execute_includesub(memory, &line)?;
                }
                TLineType::Log => {
                    let mut eater = EaterLog::new(line.clone());
                    eater.analyze(self, memory)?;
                }
                TLineType::DumpMemory => {
                    let mut eater = EaterDumpMemory::new(line.clone());
                    eater.analyze(self, memory)?;
                }
                TLineType::Option => {
                    let mut eater = EaterOption::new(line.clone());
                    eater.analyze(self, memory)?;
                }
                TLineType::Undef => {
                    let mut eater = EaterUndef::new(line.clone());
                    eater.analyze(self, memory)?;
                }
                TLineType::Startsub | TLineType::Endsub | TLineType::EndFunction | TLineType::CommentSimple | TLineType::CommentLongStart => {
                    // Handled before match or skip comments
                }
            }
            i = self.reading_index;
        }
        // Restore outer reading state.
        self.reading_index = saved_index;
        Ok(None)
    }

    fn add_plain(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        // Mirrors `TContext.addPlain`, which propagates `EaterException`:
        // a failure (e.g. an unterminated call from a backslash continuation)
        // aborts processing and lets `TimLoader` truncate output at the line.
        let new_s = self.apply_functions_and_variables_internal(memory, s)?;
        // Empty string from a non-empty input means a procedure was executed
        // and its body lines were already added to result_list. Don't push.
        if !new_s.is_empty() || s.get_string().is_empty() {
            let final_s = if let Some(pending) = self.pending_add.take() {
                format!("{pending}{new_s}")
            } else {
                new_s
            };
            self.result_list
                .push(StringLocated::new(final_s, s.get_location().clone()));
        }
        Ok(())
    }

    /// Applies function calls and variable substitutions to a line.
    ///
    /// Ported from `TContext.applyFunctionsAndVariables`.
    pub fn apply_functions_and_variables(
        &mut self,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Option<String> {
        self.apply_functions_and_variables_internal(memory, s).ok()
    }

    fn apply_functions_and_variables_internal(
        &mut self,
        memory: &mut dyn TMemory,
        s: &StringLocated,
    ) -> Result<String, EaterException> {
        let str = s.get_string();
        let mut result = String::new();
        let mut i = 0;
        let bytes: Vec<char> = str.chars().collect();

        while i < bytes.len() {
            // Try function name match at every position (Java calls getFunctionNameAt at every i)
            if let Some(func_name) = self.get_function_name_at(str, i) {
                let end = i + func_name.len();
                if end < bytes.len() && bytes[end] == '(' {
                    // Find the matching close parenthesis.
                    let mut depth = 1;
                    let mut j = end + 1;
                    let mut in_string = false;
                    let mut string_char = '\0';
                    while j < bytes.len() {
                        let c = bytes[j];
                        if in_string {
                            if c == string_char {
                                in_string = false;
                            }
                        } else if c == '"' || c == '\'' {
                            in_string = true;
                            string_char = c;
                        } else if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                j += 1;
                                break;
                            }
                        }
                        j += 1;
                    }
                    // Match the Java port: no multi-line call rescue here. An
                    // unclosed call (e.g. an unterminated string from a
                    // trailing-backslash continuation) is parsed as-is and
                    // fails inside `EaterFunctionCall`, surfacing as a
                    // preprocessor error. Backslash merging only happens in
                    // the main-source reader chain, never inside `!includesub`.
                    let func_call_str = str[i..j].to_string();
                    let func_call = StringLocated::new(func_call_str, s.get_location().clone());
                    // Set pending_add BEFORE executing the procedure so the procedure body
                    // can pick it up (text before the call is prepended to first body line).
                    let text_before = result.clone();
                    let value = self.execute_function_call(memory, &func_call, &func_name, &text_before)?;
                    let value_str = value.to_string();
                    if value_str.is_empty() {
                        // Procedure was executed: its body lines are already in result_list.
                        result.clear();
                        // Append remaining text after the procedure call to the last result line.
                        let remaining = &str[j..];
                        if !remaining.is_empty() {
                            if let Some(last) = self.result_list.last_mut() {
                                let new_str = format!("{}{}", last.get_string(), remaining);
                                *last = StringLocated::new(new_str, last.get_location().clone());
                            }
                        }
                        // Procedure: return early (the remaining text was appended).
                        return Ok(String::new());
                    }
                    result.push_str(&value_str);
                    i = j;
                    continue;
                }
            }

            // Variable substitution ($var)
            let ch = bytes[i];
            if ch == '$' {
                let var_manager = super::variable_manager::VariableManager::new(self, memory, s);
                if let Some(varname) = var_manager.get_varname_at(str, i) {
                    if let Some(value) = memory.get_variable(&varname) {
                        result.push_str(&value.to_string());
                        i += varname.len();
                        continue;
                    }
                }
            }

            result.push(ch);
            i += 1;
        }

        Ok(result)
    }

    fn execute_function_call(
        &mut self,
        memory: &mut dyn TMemory,
        s: &StringLocated,
        function_name: &str,
        text_before: &str,
    ) -> Result<TValue, EaterException> {
        let is_legacy_define = self.is_legacy_define(function_name);
        let unquoted = self.is_unquoted(function_name);
        let mut eater_call =
            super::eater_function_call::EaterFunctionCall::new(s.clone(), is_legacy_define, unquoted);
        eater_call.analyze(self, memory)?;
        let values = eater_call.get_values().to_vec();
        let named = eater_call.get_named_arguments().clone();
        let signature = TFunctionSignature::with_named_arguments(
            function_name,
            values.len() as i32,
            named.keys().cloned().collect(),
        );
        let fs = std::mem::take(&mut self.functions_set);
        // Use transmute to get a raw pointer without extending the borrow of `fs`.
        let (func_ptr, func_type) = unsafe {
            let func = fs.get_function_smart(&signature);
            func.map_or((None, super::t_function_type::TFunctionType::ReturnFunction), |f| {
                let ptr: *const dyn super::t_function::TFunction =
                    std::mem::transmute::<&dyn super::t_function::TFunction, *const dyn super::t_function::TFunction>(f);
                let ftype = f.get_function_type();
                (Some(ptr), ftype)
            })
        };
        if let Some(func_ptr) = func_ptr {
            // Restore functions_set BEFORE execution so nested calls can find functions.
            self.functions_set = fs;
            match func_type {
                super::t_function_type::TFunctionType::Procedure => {
                    // Set pending_add BEFORE executing the procedure body so the
                    // first body line picks up the text before the call.
                    self.pending_add = Some(text_before.to_string());
                    unsafe {
                        (*func_ptr).execute_procedure_internal(self, memory, s, &values, &named)?;
                    }
                    return Ok(TValue::from_string(String::new()));
                }
                _ => {
                    return unsafe {
                        (*func_ptr).execute_return_function(self, memory, s, &values, &named)
                    };
                }
            }
        }
        self.functions_set = fs;
        Err(EaterException::new(format!("No such function {function_name}"), s))
    }
    fn execute_theme(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterTheme::new(s.clone(), self.path_system.clone());
        eater.analyze(self, memory)?;
        let _theme = eater.get_theme()?;
        Ok(())
    }

    fn execute_include(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterInclude::new(s.clone());
        eater.analyze(self, memory)?;
        // Include processing would go here
        Ok(())
    }

    fn execute_include_def(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterIncludeDef::new(s.clone());
        eater.analyze(self, memory)?;
        Ok(())
    }

    fn execute_import(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterImport::new(s.clone());
        eater.analyze(self, memory)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_startsub(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterStartsub::new(s.clone());
        eater.analyze(self, memory)?;
        Ok(())
    }

    fn execute_includesub(&mut self, memory: &mut dyn TMemory, s: &StringLocated) -> Result<(), EaterException> {
        let mut eater = EaterIncludesub::new(s.clone());
        eater.analyze(self, memory)?;
        let what = eater.get_what().unwrap_or("").to_string();

        // `filename!blocname`: read the file through the same reader chain as
        // the JAR (ReadLineReader -> UncommentReadLine -> Sub::from_file).
        // Note: no ReadFilterMergeLines here, matching TContext in
        // PlantUML 1.2026.6 — trailing-backslash continuations inside an
        // included sub stay unmerged there.
        let idx = what.find('!');
        let mut sub: Option<Sub> = None;
        if let Some(idx) = idx {
            let filename = &what[..idx];
            let blocname = &what[idx + 1..];
            let saved_path_system = self.path_system.clone();
            if let Some(path) = self.path_system.resolve_file(filename) {
                if let Ok(file) = std::fs::File::open(&path) {
                    self.path_system.set_current_dir(
                        path.parent()
                            .map(std::path::Path::to_path_buf)
                            .unwrap_or_default(),
                    );
                    let reader = crate::preproc::ReadLineReader::new(
                        Box::new(std::io::BufReader::new(file)),
                        &what,
                        Some(s.get_location().clone()),
                    );
                    let mut reader = crate::preproc::UncommentReadLine::new(Box::new(reader));
                    if let Ok(found) = Sub::from_file(&mut reader, blocname) {
                        sub = found;
                    } else {
                        self.path_system = saved_path_system;
                        return Err(EaterException::new(format!("cannot include {what}"), s));
                    }
                }
            }
            // Restore the path system (mirrors the Java `finally` block),
            // even when no sub block was found below.
            self.path_system = saved_path_system;
        }

        // Plain block name (or a file that carried no matching block): look
        // the sub up among blocks collected from the current source.
        if sub.is_none() {
            if let Some(block_name) = idx.is_none().then_some(what.as_str()) {
                sub = self.subs.get(block_name).cloned();
            }
        }

        let Some(sub) = sub else {
            return Err(EaterException::new(format!("cannot include {what}"), s));
        };

        let sub_lines = sub.lines().to_vec();
        self.execute_lines_internal(memory, &sub_lines, None, false)?;
        Ok(())
    }

    /// Parses a variable as JSON, falling back to string if parsing fails.
    ///
    /// Ported from `TContext.fromJson` (Java lines 315-322).
    pub fn from_json(
        &mut self,
        memory: &mut dyn TMemory,
        name: &str,
        location: &crate::StringLocated,
    ) -> TValue {
        let s = crate::string_located::StringLocated::new(name.to_string(), location.get_location().clone());
        let result = self.apply_functions_and_variables(memory, &s);
        let result = result.unwrap_or_else(|| name.to_string());
        serde_json::from_str::<serde_json::Value>(&result).map_or_else(|_| TValue::from_string(result), TValue::from_json)
    }

    /// Creates a `Knowledge` view over this context and memory.
    ///
    /// Ported from `TContext.asKnowledge` (Java lines 298-313).
    pub fn as_knowledge<'a>(
        &mut self,
        memory: *mut (dyn TMemory + 'a),
        location: &StringLocated,
    ) -> super::expression::knowledge::ContextKnowledge<'a> {
        // SAFETY: The raw pointers are valid for the duration of expression
        // evaluation (single-threaded, stack-owned). Taking `memory` as a raw
        // pointer lets the caller keep using the original `&mut dyn TMemory`
        // while the `ContextKnowledge` exists; the memory pointer keeps the
        // original lifetime `'a`, which `get_result` outlives.
        let ctx_ptr = std::ptr::from_mut::<Self>(self);
        super::expression::knowledge::ContextKnowledge::new(ctx_ptr, memory, location.clone())
    }

    /// Extracts and joins results from position `n1` onward.
    ///
    /// Ported from `TContext.extractFromResultList`.
    pub fn extract_from_result_list(&mut self, n1: usize) -> String {
        let mut sb = String::new();
        while self.result_list.len() > n1 {
            sb.push_str(self.result_list[n1].get_string());
            self.result_list.remove(n1);
            if self.result_list.len() > n1 {
                sb.push('\n');
            }
        }
        sb
    }

    /// Appends an end-of-line to the last result.
    pub fn append_end_of_line(&mut self, end_of_line: &str) {
        if let Some(last) = self.result_list.last_mut() {
            *last = last.append(end_of_line);
        }
    }

    /// Returns the current theme metadata (stubbed).
    ///
    /// Ported from `TContext.getThemeMetadata`.
    pub fn get_theme_metadata(&self) -> String {
        String::new()
    }

    /// Returns the xargs list (stubbed).
    ///
    /// Ported from `TContext.getXargs`.
    pub fn get_xargs(&self) -> &[crate::string_located::StringLocated] {
        &[]
    }

    /// Executes a return function by signature, resolving the borrow conflict.
    ///
    /// This is used by builtins that need to call user functions.
    pub fn execute_return_function_by_signature(
        &mut self,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        signature: &TFunctionSignature,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<TValue, EaterException> {
        let fs = std::mem::take(&mut self.functions_set);
        let func = fs.get_function_smart(signature);
        if let Some(func) = func {
            let result = func.execute_return_function(self, memory, location, args, named);
            self.functions_set = fs;
            return result;
        }
        self.functions_set = fs;
        Err(EaterException::new(
            format!("Cannot find function {}", signature.get_function_name()),
            location,
        ))
    }

    /// Executes a procedure by signature, resolving the borrow conflict.
    ///
    /// This is used by builtins that need to call user procedures.
    pub fn execute_procedure_by_signature(
        &mut self,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        signature: &TFunctionSignature,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<(), EaterException> {
        let fs = std::mem::take(&mut self.functions_set);
        let func = fs.get_function_smart(signature);
        if let Some(func) = func {
            let result = func.execute_procedure_internal(self, memory, location, args, named);
            self.functions_set = fs;
            return result;
        }
        self.functions_set = fs;
        Err(EaterException::new(
            format!("Cannot find procedure {}", signature.get_function_name()),
            location,
        ))
    }
}

impl Default for TContext {
    fn default() -> Self {
        Self::new(&crate::preproc::defines::Defines::default())
    }
}

