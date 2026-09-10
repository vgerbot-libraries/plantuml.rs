//! Variable manager — handles `$variable` substitution in lines.
//!
//! Ported from `net.sourceforge.plantuml.tim.VariableManager`.

use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::{StringLocated, TLineType};

/// Manages variable substitution in preprocessor lines.
///
/// Ported from `net.sourceforge.plantuml.tim.VariableManager`.
pub struct VariableManager<'a> {
    memory: &'a mut dyn TMemory,
    context: &'a mut TContext,
    location: &'a StringLocated,
}

impl<'a> VariableManager<'a> {
    /// Creates a new `VariableManager`.
    ///
    /// Ported from `VariableManager` constructor.
    pub fn new(context: &'a mut TContext, memory: &'a mut dyn TMemory, location: &'a StringLocated) -> Self {
        Self {
            memory,
            context,
            location,
        }
    }

    /// Replaces variables in a string starting at position `i`, appending to `result`.
    /// Returns the new position after replacement.
    ///
    /// Ported from `VariableManager.replaceVariables`.
    pub fn replace_variables(
        &mut self,
        str: &str,
        i: usize,
        result: &mut String,
    ) -> Result<usize, EaterException> {
        let present_variable = match self.get_varname_at(str, i) {
            Some(v) => v,
            None => return Ok(i),
        };
        if result.ends_with("##") {
            result.truncate(result.len() - 2);
        }
        let value = self.memory.get_variable(&present_variable);
        let mut new_i = i + present_variable.len() - 1;
        if let Some(value) = value {
            if value.is_json() {
                let json = value.to_json();
                if json.is_string() {
                    if let Some(s) = json.as_str() {
                        result.push_str(s);
                    }
                } else if json.is_number() {
                    result.push_str(&json.to_string());
                } else {
                    let json_value = if json.is_array() || json.is_object() {
                        json.clone()
                    } else {
                        json.clone()
                    };
                    new_i += 1;
                    new_i = self.replace_json(json_value, str, new_i, result)? - 1;
                }
            } else {
                result.push_str(&value.to_string());
            }
        }
        if new_i + 2 < str.len() {
            let bytes = str.as_bytes();
            if bytes[new_i + 1] == b'#' && bytes[new_i + 2] == b'#' {
                new_i += 2;
            }
        }
        Ok(new_i)
    }

    fn replace_json(
        &mut self,
        mut json_value: serde_json::Value,
        str: &str,
        mut i: usize,
        result: &mut String,
    ) -> Result<usize, EaterException> {
        let bytes: Vec<char> = str.chars().collect();
        while i < bytes.len() {
            let n = bytes[i];
            if n == '.' {
                i += 1;
                let mut field_name = String::new();
                while i < bytes.len() {
                    if !is_java_identifier_part(bytes[i]) {
                        break;
                    }
                    field_name.push(bytes[i]);
                    i += 1;
                }
                if let serde_json::Value::Object(obj) = &json_value {
                    json_value = obj
                        .get(&field_name)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                }
            } else if n == '[' {
                i += 1;
                let mut in_bracket = String::new();
                let mut level = 0;
                loop {
                    if bytes[i] == '[' {
                        level += 1;
                    }
                    if bytes[i] == ']' {
                        if level == 0 {
                            break;
                        }
                        level -= 1;
                    }
                    in_bracket.push(bytes[i]);
                    i += 1;
                }
                let nb_string = self.context.apply_functions_and_variables(
                    self.memory,
                    &StringLocated::new(in_bracket, self.location.get_location().clone()),
                );
                if let Some(nb_string) = nb_string {
                    match &json_value {
                        serde_json::Value::Array(arr) => {
                            let nb: usize = nb_string.parse().unwrap_or(0);
                            json_value = arr.get(nb).cloned().unwrap_or(serde_json::Value::Null);
                        }
                        serde_json::Value::Object(obj) => {
                            json_value = obj
                                .get(&nb_string)
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                        }
                        _ => {
                            return Err(EaterException::new("Major parsing error", self.location));
                        }
                    }
                }
                i += 1;
            } else {
                break;
            }
        }
        match &json_value {
            serde_json::Value::String(s) => result.push_str(s),
            _ => result.push_str(&json_value.to_string()),
        }
        Ok(i)
    }

    /// Returns the variable name at the given position, or `None`.
    ///
    /// Ported from `VariableManager.getVarnameAt`.
    #[must_use]
    pub fn get_varname_at(&self, s: &str, pos: usize) -> Option<String> {
        let bytes: Vec<char> = s.chars().collect();
        let just_after_a_letter = pos > 0
            && TLineType::is_letter_or_emoji_or_underscore_or_digit(bytes[pos - 1])
            && !Self::just_after_backslash_n(s, pos);
        if just_after_a_letter && bytes[pos] != '$' {
            return None;
        }
        let trie = self.memory.variables_names3();
        let varname = trie.get_longuest_match_starting_in(s, pos);
        if varname.is_empty() {
            return None;
        }
        let end_pos = pos + varname.len();
        if end_pos == s.len()
            || !TLineType::is_letter_or_emoji_or_underscore_or_digit(
                s.as_bytes().get(end_pos).copied().map_or('\0', |b| b as char),
            )
        {
            return Some(varname);
        }
        None
    }

    /// Returns `true` if the position is just after a `\n` sequence.
    ///
    /// Ported from `VariableManager.justAfterBackslashN`.
    #[must_use]
    pub fn just_after_backslash_n(s: &str, pos: usize) -> bool {
        if pos <= 1 {
            return false;
        }
        let bytes = s.as_bytes();
        pos > 1 && bytes[pos - 2] == b'\\' && bytes[pos - 1] == b'n'
    }
}

fn is_java_identifier_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}
