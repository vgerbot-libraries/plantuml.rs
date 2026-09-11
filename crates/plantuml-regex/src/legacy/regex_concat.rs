//! `RegexConcat` — concatenation of regex parts with fox-signature fast reject.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexConcat` (Java).
//!
//! `RegexConcat` is a sequence of `IRegex` parts. It supports a fox-signature
//! fast-reject: if the input string doesn't contain the required character
//! classes, the match is rejected without running the regex engine.

use std::collections::HashMap;

use parking_lot::Mutex;

use super::fox_signature;
use super::iregex::IRegex;
use super::matcher_iterator::MatcherIterator;
use super::pattern2::Pattern2;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Concatenation of regex parts with fox-signature fast reject.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexConcat`.
pub struct RegexConcat {
    partials: Vec<Box<dyn IRegex>>,
    fox_regex: Mutex<Option<u64>>,
    limit_size: usize,
    full_cached: Mutex<Option<Pattern2>>,
}

impl RegexConcat {
    /// Creates a concatenation of the given parts.
    ///
    /// Ported from `RegexConcat(IRegex...)`.
    pub fn new(partials: Vec<Box<dyn IRegex>>) -> Self {
        Self {
            partials,
            fox_regex: Mutex::new(None),
            limit_size: 0,
            full_cached: Mutex::new(None),
        }
    }

    /// Builds a concatenation (factory method with key, currently unused).
    ///
    /// Ported from `RegexConcat.build(String, IRegex...)`.
    pub fn build(_key: &str, partials: Vec<Box<dyn IRegex>>) -> Self {
        Self::new(partials)
    }

    /// Sets the maximum input size for fast rejection.
    ///
    /// Ported from `RegexConcat.protectSize(int)`.
    pub const fn protect_size(mut self, size: usize) -> Self {
        self.limit_size = size;
        self
    }

    /// Computes the fox signature of the regex (OR of leaf signatures).
    ///
    /// Ported from `RegexConcat.foxRegex()`.
    fn fox_regex(&self) -> u64 {
        let mut guard = self.fox_regex.lock();
        if let Some(result) = *guard {
            return result;
        }
        // Skip first and last parts (typically start/end anchors)
        let mut tmp: u64 = 0;
        if self.partials.len() > 2 {
            for i in 1..self.partials.len() - 1 {
                let pattern = self.partials[i].get_pattern_as_string();
                tmp |= fox_signature::get_fox_signature_from_regex(&pattern);
            }
        }
        *guard = Some(tmp);
        tmp
    }

    /// Returns the compiled pattern, cached.
    fn get_pattern2(&self) -> Pattern2 {
        let mut guard = self.full_cached.lock();
        if guard.is_none() {
            *guard = Some(Pattern2::cmpile(&self.get_pattern_as_string()));
        }
        guard.as_ref().unwrap().clone()
    }
}

impl IRegex for RegexConcat {
    fn get_pattern_as_string(&self) -> String {
        self.partials
            .iter()
            .map(|p| p.get_pattern_as_string())
            .collect()
    }

    fn count(&self) -> usize {
        self.partials.iter().map(|p| p.count()).sum()
    }

    fn create_partial_match(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        let mut result = HashMap::new();
        for r in &self.partials {
            result.extend(r.create_partial_match(it));
        }
        result
    }

    fn r#match(&self, full: &str) -> bool {
        if self.limit_size != 0 && full.len() > self.limit_size {
            return false;
        }
        let fox_regex = self.fox_regex();
        if fox_regex != 0 {
            let fox_line = fox_signature::get_fox_signature_from_real_string(full);
            if fox_regex & fox_line != fox_regex {
                return false;
            }
        }
        let pattern = self.get_pattern2();
        let mut matcher = pattern.matcher(full, 0);
        matcher.find()
    }

    fn matcher(&self, full: &str) -> Option<RegexResult> {
        let pattern = self.get_pattern2();
        let mut matcher = pattern.matcher(full, 0);
        if !matcher.find() {
            return None;
        }
        let mut it = MatcherIterator::new(&matcher);
        let data = self.create_partial_match(&mut it);
        Some(RegexResult::from_data(data))
    }
}

impl std::fmt::Debug for RegexConcat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexConcat")
            .field("partials_count", &self.partials.len())
            .field("limit_size", &self.limit_size)
            .finish_non_exhaustive()
    }
}
