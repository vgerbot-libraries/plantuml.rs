//! A single `!define` macro.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Define`.

use std::sync::OnceLock;

use regex::Regex;

use crate::preproc::define_signature::DefineSignature;
use crate::stubs::BackSlash;
use crate::tim::expression::TValue;

/// A single `!define` macro, with a signature and a definition body.
///
/// Ported from `net.sourceforge.plantuml.preproc.Define`.
pub struct Define {
    signature: DefineSignature,
    definition: Option<String>,
    definition_quoted: Option<String>,
    empty_parentheses: bool,
    pattern: OnceLock<Option<Regex>>,
}

impl Define {
    /// Creates a new `Define` from a key, lines of definition, and whether empty parentheses are allowed.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Define.Define`.
    #[must_use]
    pub fn new(key: &str, lines: Option<&[String]>, empty_parentheses: bool) -> Self {
        let (definition, definition_quoted) = lines.map_or((None, None), |lines| {
            let joined = lines.join("\n");
            let quoted = regex::escape(&joined);
            (Some(joined), Some(quoted))
        });

        let signature = DefineSignature::new(key, definition_quoted.as_deref().unwrap_or(""));

        Self {
            signature,
            definition,
            definition_quoted,
            empty_parentheses,
            pattern: OnceLock::new(),
        }
    }

    /// Returns the function name of this define.
    #[must_use]
    pub fn get_function_name(&self) -> &str {
        self.signature.get_fonction_name()
    }

    /// Returns the definition as a `TValue` string.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Define.asTVariable`.
    #[must_use]
    pub fn as_t_variable(&self) -> TValue {
        TValue::from_string(self.definition.as_deref().unwrap_or(""))
    }

    /// Applies this define macro to a line, substituting the macro call with its definition.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Define.apply`.
    pub fn apply(&self, line: &str) -> String {
        if self.definition.is_none() {
            return line.to_string();
        }

        if !line.contains(self.get_function_name()) {
            return line.to_string();
        }

        if self.signature.is_method() {
            if !line.contains('(') {
                return line.to_string();
            }
            self.apply1(line)
        } else {
            self.apply2(line)
        }
    }

    /// Non-method substitution: replace the bare keyword with the definition.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Define.apply2`.
    fn apply2(&self, line: &str) -> String {
        let pattern = self.pattern.get_or_init(|| {
            let regex_str = format!(
                r"\b{}\b{}",
                regex::escape(self.signature.get_key()),
                if self.empty_parentheses { r"(\(\))?" } else { "" }
            );
            Regex::new(&regex_str).ok()
        });

        let line = BackSlash::translate_back_slashes(line);
        let line = if let Some(ref pat) = pattern {
            let quoted = self.definition_quoted.as_deref().unwrap_or("");
            pat.replace_all(&line, quoted).to_string()
        } else {
            line
        };
        BackSlash::untranslate_back_slashes(&line)
    }

    /// Method substitution: apply each variable variation.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Define.apply1`.
    fn apply1(&self, line: &str) -> String {
        let mut result = line.to_string();
        for vars in self.signature.get_variation_variables() {
            result = vars.apply_on(&result);
        }
        result
    }
}

impl Clone for Define {
    fn clone(&self) -> Self {
        Self {
            signature: self.signature.clone(),
            definition: self.definition.clone(),
            definition_quoted: self.definition_quoted.clone(),
            empty_parentheses: self.empty_parentheses,
            pattern: OnceLock::new(),
        }
    }
}

impl std::fmt::Display for Define {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature)
    }
}
