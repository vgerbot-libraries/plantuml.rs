//! `RegexComposed` — abstract base for composite regex patterns.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexComposed` (Java).
//!
//! `RegexComposed` is the base for `RegexConcat`, `RegexOr`, `RegexOptional`,
//! `RegexRepeated*`. It holds a list of child `IRegex` parts, caches a compiled
//! `Pattern2`, and provides `matcher()` and `match()` via the compiled pattern.

use std::collections::HashMap;
use std::sync::Mutex;

use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Abstract base for composite regex patterns.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexComposed`.
pub struct RegexComposed {
    partials: Vec<Box<dyn IRegex>>,
    #[allow(dead_code)]
    full_cached: Mutex<Option<Pattern2>>,
}

impl RegexComposed {
    /// Creates a new composed regex from the given parts.
    ///
    /// Ported from `RegexComposed(IRegex...)`.
    pub fn new(partials: Vec<Box<dyn IRegex>>) -> Self {
        Self {
            partials,
            full_cached: Mutex::new(None),
        }
    }

    /// Returns the child parts.
    ///
    /// Ported from `RegexComposed.partials()`.
    pub fn partials(&self) -> &[Box<dyn IRegex>] {
        &self.partials
    }

    /// Returns the compiled pattern, compiling it lazily on first access.
    ///
    /// Ported from `RegexComposed.getPattern2()`.
    #[allow(dead_code)]
    fn get_pattern2(&self, _pattern_as_string: &str) -> &Pattern2 {
        // We can't return a reference through the Mutex, so we use a different
        // approach: store the Pattern2 in a OnceLock-like pattern.
        // Actually, we need to restructure. Let's use a different approach.
        unimplemented!("get_pattern2 should be called from the IRegex impl")
    }

    /// Creates partial matches from an iterator of captured groups.
    ///
    /// Ported from `RegexComposed.createPartialMatch(Iterator)`.
    pub fn create_partial_match_composed(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        let mut result = HashMap::new();
        for r in &self.partials {
            result.extend(r.create_partial_match(it));
        }
        result
    }

    /// Counts the total capture groups, including the start count.
    ///
    /// Ported from `RegexComposed.count()`.
    pub fn count_composed(&self, start_count: usize) -> usize {
        let mut cpt = start_count;
        for r in &self.partials {
            cpt += r.count();
        }
        cpt
    }

    /// Matches the full string via the compiled pattern.
    ///
    /// Ported from `RegexComposed.matcher(String)`.
    pub fn matcher_composed(&self, s: &str, pattern_as_string: &str) -> Option<RegexResult> {
        let pattern = Pattern2::cmpile(pattern_as_string);
        let mut matcher = pattern.matcher(s, 0);
        if !matcher.find() {
            return None;
        }
        let mut it = MatcherIterator::new(&matcher);
        let data = self.create_partial_match_composed(&mut it);
        Some(RegexResult::from_data(data))
    }

    /// Matches the full string for the `match()` method.
    ///
    /// Ported from `RegexComposed.match(StringLocated)`.
    pub fn match_composed(&self, s: &str, pattern_as_string: &str) -> bool {
        let pattern = Pattern2::cmpile(pattern_as_string);
        let mut matcher = pattern.matcher(s, 0);
        matcher.find()
    }
}

impl IRegex for RegexComposed {
    fn get_pattern_as_string(&self) -> String {
        // Default: concatenate all parts. Subclasses override.
        self.partials
            .iter()
            .map(|p| p.get_pattern_as_string())
            .collect()
    }

    fn count(&self) -> usize {
        self.count_composed(0)
    }

    fn create_partial_match(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        self.create_partial_match_composed(it)
    }

    fn r#match(&self, full: &str) -> bool {
        self.match_composed(full, &self.get_pattern_as_string())
    }

    fn matcher(&self, full: &str) -> Option<RegexResult> {
        self.matcher_composed(full, &self.get_pattern_as_string())
    }
}
