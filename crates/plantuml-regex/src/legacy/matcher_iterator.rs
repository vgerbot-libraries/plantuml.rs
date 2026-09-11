//! Iterator over captured groups of a `Matcher2`.
//!
//! Ported from `net.sourceforge.plantuml.regex.MatcherIterator` (Java).
//!
//! Iterates groups from 1 to `groupCount()`, returning each group as an
//! `Option<String>` (`None` for non-participating groups, matching Java's
//! `null`).

use super::matcher2::Matcher2;

/// Iterator over regex capture groups, starting at group 1.
///
/// Ported from `net.sourceforge.plantuml.regex.MatcherIterator`.
pub struct MatcherIterator<'a> {
    cpt: usize,
    matcher: &'a Matcher2<'a>,
}

impl<'a> MatcherIterator<'a> {
    /// Creates an iterator over the matcher's capture groups.
    ///
    /// Ported from `MatcherIterator(Matcher2)`.
    pub const fn new(matcher: &'a Matcher2<'a>) -> Self {
        Self { cpt: 1, matcher }
    }
}

impl Iterator for MatcherIterator<'_> {
    type Item = Option<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cpt > self.matcher.group_count() {
            return None;
        }
        let group = self.matcher.group(self.cpt);
        self.cpt += 1;
        Some(group)
    }
}
