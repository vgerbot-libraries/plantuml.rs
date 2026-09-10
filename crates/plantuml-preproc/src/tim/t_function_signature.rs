//! Function signature for preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.TFunctionSignature`.

use std::collections::HashSet;

/// The signature of a preprocessor function (name, arg count, named arguments).
///
/// Ported from `net.sourceforge.plantuml.tim.TFunctionSignature`.
#[derive(Debug, Clone)]
pub struct TFunctionSignature {
    function_name: String,
    nb_arg: i32,
    named_arguments: HashSet<String>,
    cached_hash: Option<u64>,
}

impl TFunctionSignature {
    /// Creates a new `TFunctionSignature` with the given name and arg count.
    #[must_use]
    pub fn new(function_name: impl Into<String>, nb_arg: i32) -> Self {
        Self::with_named_arguments(function_name, nb_arg, HashSet::new())
    }

    /// Creates a new `TFunctionSignature` with named arguments.
    #[must_use]
    pub fn with_named_arguments(
        function_name: impl Into<String>,
        nb_arg: i32,
        named_arguments: HashSet<String>,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            nb_arg,
            named_arguments,
            cached_hash: None,
        }
    }

    /// Returns `true` if this signature has the same function name as another.
    ///
    /// Ported from `TFunctionSignature.sameFunctionNameAs`.
    #[must_use]
    pub fn same_function_name_as(&self, other: &Self) -> bool {
        self.function_name == other.function_name
    }

    /// Returns the function name.
    #[must_use]
    pub fn get_function_name(&self) -> &str {
        &self.function_name
    }

    /// Returns the number of arguments.
    #[must_use]
    pub fn get_nb_arg(&self) -> i32 {
        self.nb_arg
    }

    /// Returns the named arguments.
    #[must_use]
    pub fn get_named_arguments(&self) -> &HashSet<String> {
        &self.named_arguments
    }
}

impl PartialEq for TFunctionSignature {
    fn eq(&self, other: &Self) -> bool {
        self.function_name == other.function_name && self.nb_arg == other.nb_arg
    }
}

impl Eq for TFunctionSignature {}

impl std::hash::Hash for TFunctionSignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(h) = self.cached_hash {
            h.hash(state);
            return;
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.function_name.hash(&mut hasher);
        self.nb_arg.hash(&mut hasher);
        let result = std::hash::Hasher::finish(&hasher);
        result.hash(state);
    }
}

impl std::fmt::Display for TFunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} {:?}", self.function_name, self.nb_arg, self.named_arguments)
    }
}
