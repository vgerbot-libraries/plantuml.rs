//! Numeric comparison operator.
//!
//! Ported from `net.sourceforge.plantuml.preproc.NumericCompare`.

/// Compares two integers using a string operator like `<`, `<=`, `>`, `>=`, `=`, `==`, `!=`, `<>`.
///
/// Ported from `net.sourceforge.plantuml.preproc.NumericCompare`.
pub struct NumericCompare {
    operator: String,
}

impl NumericCompare {
    /// Creates a new `NumericCompare` with the given operator.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.NumericCompare.NumericCompare`.
    #[must_use]
    pub fn new(operator: &str) -> Self {
        Self {
            operator: operator.to_string(),
        }
    }

    /// Returns `true` if the comparison holds for the two values.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.NumericCompare.isCompareOk`.
    #[must_use]
    pub fn is_compare_ok(&self, value1: i32, value2: i32) -> bool {
        match self.operator.as_str() {
            "<" => value1 < value2,
            "<=" => value1 <= value2,
            ">" => value1 > value2,
            ">=" => value1 >= value2,
            "=" | "==" => value1 == value2,
            "!=" | "<>" => value1 != value2,
            _ => false,
        }
    }
}
