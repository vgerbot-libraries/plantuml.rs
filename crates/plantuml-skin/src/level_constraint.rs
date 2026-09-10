//! LevelConstraint — the `depth(n)` / `*` side of a style declaration or query.
//!
//! Ported from: `net/sourceforge/plantuml/style/parser2/LevelConstraint.java`

/// Depth/star constraint on a style declaration or query.
///
/// Ported from: `net/sourceforge/plantuml/style/parser2/LevelConstraint.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelConstraint {
    level: i32,
    star: bool,
}

/// Sentinel for "no level set".
pub const NO_LEVEL: i32 = -1;

impl LevelConstraint {
    /// No `depth(n)` constraint at all: matches at any level (or no level).
    #[must_use]
    pub fn none() -> Self {
        Self {
            level: NO_LEVEL,
            star: false,
        }
    }

    /// Creates a constraint with the given level and star flag.
    #[must_use]
    pub fn of(level: i32, star: bool) -> Self {
        if level == NO_LEVEL {
            if star {
                Self {
                    level: NO_LEVEL,
                    star: true,
                }
            } else {
                Self::none()
            }
        } else {
            Self { level, star }
        }
    }

    #[must_use]
    pub fn get_level(self) -> i32 {
        self.level
    }

    #[must_use]
    pub fn has_level(self) -> bool {
        self.level != NO_LEVEL
    }

    #[must_use]
    pub fn is_star(self) -> bool {
        self.star
    }

    /// The set of query levels this constraint, read as a declaration, accepts.
    fn accepted_levels_mask(self) -> u64 {
        if !self.has_level() {
            return u64::MAX;
        }
        if self.star {
            u64::MAX.wrapping_shl(self.level as u32)
        } else {
            1u64.wrapping_shl(self.level as u32)
        }
    }

    /// True if `declaration` (as a stored declaration) matches `query` (as the
    /// element/ancestor lookup being resolved).
    #[must_use]
    pub fn matches(declaration: Self, query: Self) -> bool {
        if declaration.has_level() {
            if !query.has_level() {
                return false;
            }
            if declaration.accepted_levels_mask() & (1u64.wrapping_shl(query.level as u32)) == 0 {
                return false;
            }
        }
        if query.star && !declaration.star {
            return false;
        }
        true
    }
}

impl std::fmt::Display for LevelConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.has_level() {
            return if self.star {
                write!(f, "*")
            } else {
                write!(f, "")
            };
        }
        write!(f, "depth({})", self.level)?;
        if self.star {
            write!(f, "*")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_constraint() {
        let c = LevelConstraint::none();
        assert!(!c.has_level());
        assert!(!c.is_star());
    }

    #[test]
    fn exact_level_match() {
        let decl = LevelConstraint::of(3, false);
        let query = LevelConstraint::of(3, false);
        assert!(LevelConstraint::matches(decl, query));
    }

    #[test]
    fn star_matches_deeper() {
        let decl = LevelConstraint::of(2, true);
        let query = LevelConstraint::of(5, false);
        assert!(LevelConstraint::matches(decl, query));
    }

    #[test]
    fn star_query_requires_star_declaration() {
        let decl = LevelConstraint::of(2, false);
        let query = LevelConstraint::of(2, true);
        assert!(!LevelConstraint::matches(decl, query));
    }
}
