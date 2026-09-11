//! `RegexRepeatedOneOrMore` — repeated pattern `(?:p)+`.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexRepeatedOneOrMore` (Java).

use std::collections::HashMap;

use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Repeated one-or-more regex pattern.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexRepeatedOneOrMore`.
pub struct RegexRepeatedOneOrMore {
    name: Option<String>,
    partial: Box<dyn IRegex>,
}

impl RegexRepeatedOneOrMore {
    /// Creates a named one-or-more repetition.
    ///
    /// Ported from `RegexRepeatedOneOrMore(String, IRegex)`.
    pub fn new(name: Option<&str>, partial: Box<dyn IRegex>) -> Self {
        Self {
            name: name.map(std::string::ToString::to_string),
            partial,
        }
    }
}

impl IRegex for RegexRepeatedOneOrMore {
    fn get_pattern_as_string(&self) -> String {
        format!("({})+", self.partial.get_pattern_as_string())
    }

    fn count(&self) -> usize {
        // start count is 1 (the outer capture group)
        1 + self.partial.count()
    }

    fn create_partial_match(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        let mut result = HashMap::new();
        let full_group = if self.name.is_some() {
            it.next().flatten()
        } else {
            None
        };
        result.extend(self.partial.create_partial_match(it));
        if let Some(name) = &self.name {
            let mut m = RegexPartialMatch::new(name);
            m.add(full_group);
            result.insert(name.clone(), m);
        }
        result
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

impl std::fmt::Debug for RegexRepeatedOneOrMore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexRepeatedOneOrMore")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}
