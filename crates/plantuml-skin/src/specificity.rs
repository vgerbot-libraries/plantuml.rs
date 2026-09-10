//! Specificity — CSS-like specificity ordering for style declarations.
//!
//! Ported from: `net/sourceforge/plantuml/style/Specificity.java`

/// How specific a style declaration is, modeled as an ordered tuple compared
/// one component at a time from most to least significant.
///
/// Ported from: `net/sourceforge/plantuml/style/Specificity.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Specificity {
    forced: bool,
    ancestor_cascade: bool,
    ancestor_rank: i32,
    stereotype_count: i32,
    order: i32,
}

impl Specificity {
    /// A plain declaration's specificity at the position it was parsed.
    #[must_use]
    pub fn at_order(order: i32) -> Self {
        Self {
            forced: false,
            ancestor_cascade: false,
            ancestor_rank: 0,
            stereotype_count: 0,
            order,
        }
    }

    /// An explicit, programmatic override that always wins.
    #[must_use]
    pub fn forced_override() -> Self {
        Self {
            forced: true,
            ancestor_cascade: false,
            ancestor_rank: 0,
            stereotype_count: 0,
            order: 0,
        }
    }

    /// This specificity, but requiring `count` stereotypes.
    #[must_use]
    pub fn with_stereotype_count(self, count: i32) -> Self {
        Self {
            stereotype_count: count,
            ..self
        }
    }

    /// This specificity, but applied through the ancestor-inheritance cascade
    /// at ancestor rank `rank` (0 = own level, more negative = farther up).
    #[must_use]
    pub fn with_ancestor_rank(self, rank: i32) -> Self {
        Self {
            ancestor_cascade: true,
            ancestor_rank: rank,
            ..self
        }
    }

    /// Whether the declaration requires at least one stereotype.
    #[must_use]
    pub fn has_stereotype(self) -> bool {
        self.stereotype_count > 0
    }

    /// Returns `true` if this specificity is strictly greater than `other`.
    #[must_use]
    pub fn is_bigger_than(self, other: Self) -> bool {
        self.cmp(&other).is_gt()
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.forced
            .cmp(&other.forced)
            .then(self.ancestor_cascade.cmp(&other.ancestor_cascade))
            .then(self.ancestor_rank.cmp(&other.ancestor_rank))
            .then(self.stereotype_count.cmp(&other.stereotype_count))
            .then(self.order.cmp(&other.order))
    }
}

impl std::fmt::Display for Specificity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.forced {
            write!(f, "override")
        } else if self.ancestor_cascade {
            write!(
                f,
                "{},stereo={},ancestor={}",
                self.order, self.stereotype_count, self.ancestor_rank
            )
        } else if self.stereotype_count != 0 {
            write!(f, "{},stereo={}", self.order, self.stereotype_count)
        } else {
            write!(f, "{}", self.order)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_comparison() {
        let a = Specificity::at_order(1);
        let b = Specificity::at_order(2);
        assert!(b.is_bigger_than(a));
        assert!(!a.is_bigger_than(b));
    }

    #[test]
    fn forced_override_wins() {
        let forced = Specificity::forced_override();
        let plain = Specificity::at_order(999);
        assert!(forced.is_bigger_than(plain));
    }

    #[test]
    fn ancestor_cascade_beats_plain() {
        let cascaded = Specificity::at_order(1).with_ancestor_rank(0);
        let plain = Specificity::at_order(999);
        assert!(cascaded.is_bigger_than(plain));
    }

    #[test]
    fn stereotype_count_matters() {
        let s1 = Specificity::at_order(1).with_stereotype_count(1);
        let s2 = Specificity::at_order(1).with_stereotype_count(2);
        assert!(s2.is_bigger_than(s1));
    }
}
