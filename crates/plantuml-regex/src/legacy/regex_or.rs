//! `RegexOr` — alternation of regex patterns.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexOr` (Java).
//!
//! `RegexOr` is an alternation: `(?:p1|p2|...|pN)`. If a name is provided,
//! the entire match is captured as a named group.

use std::collections::HashMap;

use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Alternation of regex patterns.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexOr`.
pub struct RegexOr {
    name: Option<String>,
    partials: Vec<Box<dyn IRegex>>,
}

impl RegexOr {
    /// Creates an unnamed alternation.
    ///
    /// Ported from `RegexOr(IRegex...)`.
    pub fn new(partials: Vec<Box<dyn IRegex>>) -> Self {
        Self { name: None, partials }
    }

    /// Creates a named alternation. The entire match is captured as a group.
    ///
    /// Ported from `RegexOr(String, IRegex...)`.
    pub fn with_name(name: &str, partials: Vec<Box<dyn IRegex>>) -> Self {
        Self {
            name: Some(name.to_string()),
            partials,
        }
    }
}

impl IRegex for RegexOr {
    fn get_pattern_as_string(&self) -> String {
        let mut sb = String::from("(");
        if self.name.is_none() {
            sb.push_str("?:");
        }
        for (i, p) in self.partials.iter().enumerate() {
            if i > 0 {
                sb.push('|');
            }
            sb.push_str(&p.get_pattern_as_string());
        }
        sb.push(')');
        sb
    }

    fn count(&self) -> usize {
        // start count is 1 if named
        let start = usize::from(self.name.is_some());
        start + self.partials.iter().map(|p| p.count()).sum::<usize>()
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
        for r in &self.partials {
            result.extend(r.create_partial_match(it));
        }
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

impl std::fmt::Debug for RegexOr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexOr")
            .field("name", &self.name)
            .field("partials_count", &self.partials.len())
            .finish()
    }
}
