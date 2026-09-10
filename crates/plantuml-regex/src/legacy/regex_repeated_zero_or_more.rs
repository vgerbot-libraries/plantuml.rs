//! RegexRepeatedZeroOrMore — repeated pattern `(?:p)*`.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexRepeatedZeroOrMore` (Java).

use std::collections::HashMap;

use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Repeated zero-or-more regex pattern.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexRepeatedZeroOrMore`.
pub struct RegexRepeatedZeroOrMore {
    partial: Box<dyn IRegex>,
}

impl RegexRepeatedZeroOrMore {
    /// Creates a zero-or-more repetition.
    ///
    /// Ported from `RegexRepeatedZeroOrMore(IRegex)`.
    pub fn new(partial: Box<dyn IRegex>) -> Self {
        Self { partial }
    }
}

impl IRegex for RegexRepeatedZeroOrMore {
    fn get_pattern_as_string(&self) -> String {
        format!("(?:{})*", self.partial.get_pattern_as_string())
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

impl std::fmt::Debug for RegexRepeatedZeroOrMore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexRepeatedZeroOrMore").finish()
    }
}
