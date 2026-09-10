//! Signature for a `!define` macro.
//!
//! Ported from `net.sourceforge.plantuml.preproc.DefineSignature`.

use crate::preproc::variables::Variables;

/// The signature of a `!define` macro, including the function name and parameters.
///
/// Ported from `net.sourceforge.plantuml.preproc.DefineSignature`.
pub struct DefineSignature {
    key: String,
    fonction_name: String,
    variables: Vec<Variables>,
    is_method: bool,
}

impl DefineSignature {
    /// Creates a new `DefineSignature` from a key string and quoted definition.
    ///
    /// The key is the full signature, e.g. `myfunc($a, $b)` or `MY_CONST`.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DefineSignature.DefineSignature`.
    #[must_use]
    pub fn new(key: &str, definition_quoted: &str) -> Self {
        let is_method = key.contains('(');

        // Tokenize by (, )
        let mut tokens = Vec::new();
        let mut current = String::new();
        for c in key.chars() {
            if c == '(' || c == ')' || c == ',' {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                tokens.push(c.to_string());
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() {
            tokens.push(current);
        }

        let fonction_name = tokens.first().map_or(String::new(), |t| t.trim().to_string());
        let mut master = Variables::new(&fonction_name, definition_quoted);

        for token in tokens.iter().skip(1) {
            let trimmed = token.trim();
            if trimmed == "(" || trimmed == ")" || trimmed == "," {
                continue;
            }
            master.add(crate::preproc::define_variable::DefineVariable::new(trimmed));
        }

        let count = master.count_default_value();
        let mut variables = Vec::new();
        for i in 0..=count {
            variables.push(master.remove_some_default_values(i));
        }

        Self {
            key: key.to_string(),
            fonction_name,
            variables,
            is_method,
        }
    }

    /// Returns `true` if this signature is a method (has parentheses).
    #[must_use]
    pub fn is_method(&self) -> bool {
        self.is_method
    }

    /// Returns the full key string.
    #[must_use]
    pub fn get_key(&self) -> &str {
        &self.key
    }

    /// Returns the function name (the part before the parentheses).
    #[must_use]
    pub fn get_fonction_name(&self) -> &str {
        &self.fonction_name
    }

    /// Returns the list of variable variations (for default-value combinations).
    pub fn get_variation_variables(&self) -> &[Variables] {
        &self.variables
    }
}

impl Clone for DefineSignature {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            fonction_name: self.fonction_name.clone(),
            variables: self.variables.clone(),
            is_method: self.is_method,
        }
    }
}

impl std::fmt::Display for DefineSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.key, self.fonction_name)
    }
}
