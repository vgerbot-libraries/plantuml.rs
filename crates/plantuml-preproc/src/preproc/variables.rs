//! A collection of define variables for a single macro signature.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Variables`.

use std::sync::OnceLock;

use regex::Regex;

use crate::preproc::define_variable::DefineVariable;

/// A collection of `DefineVariable`s for a single `!define` macro signature.
///
/// Generates regex patterns to match macro calls and substitute variables.
///
/// Ported from `net.sourceforge.plantuml.preproc.Variables`.
pub struct Variables {
    all: Vec<DefineVariable>,
    fonction_name: String,
    definition_quoted: String,
    /// Lazily computed regex and replacement string.
    computed: OnceLock<(Regex, String)>,
}

impl Variables {
    /// Creates a new `Variables` with the given function name and quoted definition.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Variables.Variables`.
    #[must_use]
    pub fn new(fonction_name: &str, definition_quoted: &str) -> Self {
        Self {
            all: Vec::new(),
            fonction_name: fonction_name.to_string(),
            definition_quoted: definition_quoted.to_string(),
            computed: OnceLock::new(),
        }
    }

    /// Adds a variable to this collection.
    pub fn add(&mut self, var: DefineVariable) {
        self.all.push(var);
    }

    /// Counts how many variables have default values.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Variables.countDefaultValue`.
    pub fn count_default_value(&self) -> usize {
        self.all
            .iter()
            .filter(|v| v.get_default_value().is_some())
            .count()
    }

    /// Creates a new `Variables` with `nb` default-valued variables converted to non-default.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Variables.removeSomeDefaultValues`.
    #[must_use]
    pub fn remove_some_default_values(&self, mut nb: usize) -> Self {
        let mut result = Variables::new(&self.fonction_name, &self.definition_quoted);
        for v in &self.all {
            if v.get_default_value().is_some() && nb > 0 {
                result.add(v.remove_default());
                nb -= 1;
            } else {
                result.add(v.clone());
            }
        }
        result
    }

    /// Applies the variable substitution to a line.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Variables.applyOn`.
    pub fn apply_on(&self, line: &str) -> String {
        let (regex, new_value) = self.computed.get_or_init(|| self.compute_regex_and_replacement());
        regex.replace_all(line, new_value.as_str()).to_string()
    }

    /// Computes the regex pattern and replacement string from the variables.
    fn compute_regex_and_replacement(&self) -> (Regex, String) {
        let mut new_value = self.definition_quoted.clone();
        let mut regex = format!(r"\b{}\(", regex::escape(&self.fonction_name));

        let mut appended = false;
        for (j, variable) in self.all.iter().enumerate() {
            let var_name = variable.get_name();
            let var2 = format!(
                r"(##{}##)|(##{}\b)|(\b{}##)|(\b{}\b)",
                var_name, var_name, var_name, var_name
            );
            let var_re = match Regex::new(&var2) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if variable.get_default_value().is_none() {
                regex.push_str(
                    r#"(?:(?:\s*"([^"]*)"\s*)|(?:\s*'([^']*)'\s*)|\s*((?:\([^()]*\)|[^,'"])*?))"#,
                );
                let i = 1 + 3 * j;
                let replacement = format!("${}${}${}", i, i + 1, i + 2);
                new_value = var_re.replace_all(&new_value, replacement.as_str()).to_string();
                regex.push(',');
                appended = true;
            } else if let Some(default) = variable.get_default_value() {
                let escaped = regex::escape(default);
                new_value = var_re.replace_all(&new_value, escaped.as_str()).to_string();
            }
        }

        if appended {
            regex.pop(); // Remove trailing comma
        }
        regex.push_str(r"\)");

        let compiled = Regex::new(&regex).unwrap_or_else(|_| Regex::new("$").unwrap_or_else(|_| Regex::new("$").unwrap()));
        (compiled, new_value)
    }
}

impl Clone for Variables {
    fn clone(&self) -> Self {
        Self {
            all: self.all.clone(),
            fonction_name: self.fonction_name.clone(),
            definition_quoted: self.definition_quoted.clone(),
            computed: OnceLock::new(),
        }
    }
}
