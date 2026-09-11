//! `RegexOptional` — optional regex pattern `(?:p)?`.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexOptional` (Java).

use std::collections::HashMap;

use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Optional regex pattern.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexOptional`.
pub struct RegexOptional {
    partial: Box<dyn IRegex>,
}

impl RegexOptional {
    /// Creates an optional pattern wrapping the given regex.
    ///
    /// Ported from `RegexOptional(IRegex)`.
    pub fn new(partial: Box<dyn IRegex>) -> Self {
        Self { partial }
    }
}

impl IRegex for RegexOptional {
    fn get_pattern_as_string(&self) -> String {
        format!("(?:{})?", self.partial.get_pattern_as_string())
    }

    fn count(&self) -> usize {
        self.partial.count()
    }

    fn create_partial_match(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        self.partial.create_partial_match(it)
    }

    fn r#match(&self, full: &str) -> bool {
        let pattern = Pattern2::cmpile(&self.get_pattern_as_string());
        let mut matcher = pattern.matcher(full, 0);
        matcher.find()
    }

    fn matcher(&self, full: &str) -> Option<RegexResult> {
        let pattern = Pattern2::cmpile(&self.get_pattern_as_string());
        let mut matcher = pattern.matcher(full, 0);
        if !matcher.find() {
            return None;
        }
        let mut it = MatcherIterator::new(&matcher);
        let data = self.create_partial_match(&mut it);
        Some(RegexResult::from_data(data))
    }
}

impl std::fmt::Debug for RegexOptional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexOptional").finish()
    }
}
