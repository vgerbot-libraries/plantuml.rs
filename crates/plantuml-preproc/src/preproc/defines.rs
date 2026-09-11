//! Container for `!define` macros.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Defines`.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::preproc::define::Define;
use crate::preproc::eval_boolean::EvalBoolean;
use crate::preproc::truth::Truth;
use crate::stubs::version_string;

static DATE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(clippy::trivial_regex)]
    Regex::new(r"(?i)%date(\[(.+?)\])?%").unwrap_or_else(|_| Regex::new("$").unwrap_or_else(|_| Regex::new("$").unwrap()))
});

/// Storage for `!define` macros and environment variables.
///
/// Ported from `net.sourceforge.plantuml.preproc.Defines`.
pub struct Defines {
    environment: HashMap<String, String>,
    values: HashMap<String, Define>,
}

impl Default for Defines {
    fn default() -> Self {
        Self::new()
    }
}

impl Defines {
    /// Creates a new `Defines` with the `PlantUML` version pre-set.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Defines.Defines`.
    #[must_use]
    pub fn new() -> Self {
        let mut environment = HashMap::new();
        environment.insert("PLANTUML_VERSION".to_string(), version_string());
        Self {
            environment,
            values: HashMap::new(),
        }
    }

    /// Creates an empty `Defines`.
    #[must_use]
    pub fn create_empty() -> Self {
        Self::new()
    }

    /// Returns the environment value for the given key.
    #[must_use]
    pub fn get_environment_value(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(String::as_str)
    }

    /// Overrides the filename environment variables.
    pub fn override_filename(&mut self, filename: &str) {
        self.environment
            .insert("filename".to_string(), filename.to_string());
        self.environment.insert(
            "filenameNoExtension".to_string(),
            Self::name_no_extension(filename),
        );
    }

    /// Overrides the directory path environment variable.
    pub fn override_dir_path(&mut self, file_dir: &str) {
        self.environment
            .insert("dirpath".to_string(), file_dir.replace('\\', "/"));
    }

    /// Imports all environment from another `Defines`.
    ///
    /// Note: `Define` objects cannot be cloned (they contain `OnceLock`),
    /// so only environment variables are imported.
    pub fn import_from(&mut self, other: &Self) {
        for (k, v) in &other.environment {
            self.environment.insert(k.clone(), v.clone());
        }
    }

    /// Defines a new macro.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Defines.define`.
    pub fn define(&mut self, name: &str, value: Option<&[String]>, empty_parentheses: bool) {
        self.values
            .insert(name.to_string(), Define::new(name, value, empty_parentheses));
    }

    /// Returns `true` if the given expression evaluates to true.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Defines.isDefine`.
    pub fn is_define(&self, expression: &str) -> bool {
        let eval = EvalBoolean::new(expression, self);
        eval.eval().unwrap_or(false)
    }

    /// Removes a define by name.
    pub fn undefine(&mut self, name: &str) {
        self.values.remove(name);
    }

    /// Applies all defines to a line, returning the resulting lines (split by newline).
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Defines.applyDefines`.
    pub fn apply_defines(&self, line: &str) -> Vec<String> {
        let line = self.manage_date(line);
        let line = self.manage_environment(&line);
        let line = self.method1(&line);
        line.split('\n').map(String::from).collect()
    }

    /// Applies each define to the line in order.
    fn method1(&self, line: &str) -> String {
        let mut result = line.to_string();
        for def in self.values.values() {
            result = def.apply(&result);
        }
        result
    }

    /// Replaces `%key%` patterns with environment values.
    fn manage_environment(&self, line: &str) -> String {
        let mut result = line.to_string();
        for (key, value) in &self.environment {
            let pattern = format!("%{key}%");
            result = result.replace(&pattern, value);
        }
        result
    }

    /// Replaces `%date` or `%date[format]%` with the current date.
    #[allow(clippy::unused_self)]
    fn manage_date(&self, line: &str) -> String {
        if !DATE_PATTERN.is_match(line) {
            return line.to_string();
        }
        let Some(caps) = DATE_PATTERN.captures(line) else {
            return line.to_string();
        };
        let format = caps.get(2).map(|m| m.as_str());
        let replace = format.map_or_else(Self::current_date_string, |fmt| format!("(BAD DATE PATTERN:{fmt})"));
        DATE_PATTERN.replace_all(line, replace.as_str()).to_string()
    }

    fn current_date_string() -> String {
        // Simple date string without external dependencies.
        // Java uses `new Date().toString()` which produces something like
        // "Wed Sep 09 14:30:00 UTC 2026".
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("<date {secs}>")
    }

    fn name_no_extension(name: &str) -> String {
        name.rfind('.').map_or_else(|| name.to_string(), |idx| name[..idx].to_string())
    }
}

impl Truth for Defines {
    fn is_true(&self, name: &str) -> bool {
        for key in self.values.keys() {
            if key == name || key.starts_with(&format!("{name}(")) {
                return true;
            }
        }
        false
    }
}

impl std::fmt::Display for Defines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?}",
            self.values.keys().collect::<Vec<_>>(),
            self.environment.keys().collect::<Vec<_>>()
        )
    }
}

impl Clone for Defines {
    fn clone(&self) -> Self {
        Self {
            environment: self.environment.clone(),
            values: self.values.clone(),
        }
    }
}
