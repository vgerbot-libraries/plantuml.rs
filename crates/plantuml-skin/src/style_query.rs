//! StyleQuery — the element's tags (SName set + stereotypes) and level constraint.
//!
//! Ported from: `net/sourceforge/plantuml/style/parser2/StyleQuery.java`

use std::collections::BTreeSet;

use crate::level_constraint::LevelConstraint;
use crate::s_name::SName;

/// An atom in a style query: either a named selector (`SName`) or a stereotype.
///
/// Ported from: `net/sourceforge/plantuml/style/parser2/StyleAtom.java`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleAtom {
    /// A named selector tag (e.g. `SName::Root`, `SName::Document`).
    Name(SName),
    /// A stereotype label (lowercased).
    Stereotype(String),
}

impl StyleAtom {
    /// Creates a named-selector atom.
    #[must_use]
    pub fn of(name: SName) -> Self {
        Self::Name(name)
    }

    /// Creates a stereotype atom, cleaning the label to lowercase.
    #[must_use]
    pub fn of_stereotype(stereotype: &str) -> Self {
        Self::Stereotype(stereotype.to_lowercase())
    }

    /// Returns `true` if this is a named-selector atom.
    #[must_use]
    pub fn is_name(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Returns the stereotype string if this is a stereotype atom.
    #[must_use]
    pub fn get_stereotype(&self) -> Option<&str> {
        if let Self::Stereotype(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

impl Ord for StyleAtom {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Name(a), Self::Name(b)) => a.cmp(b),
            (Self::Stereotype(a), Self::Stereotype(b)) => a.cmp(b),
            (Self::Name(_), Self::Stereotype(_)) => std::cmp::Ordering::Less,
            (Self::Stereotype(_), Self::Name(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for StyleAtom {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// What a `StyleAtomTrie` is asked to resolve: the element's tags and level
/// constraint.
///
/// Ported from: `net/sourceforge/plantuml/style/parser2/StyleQuery.java`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleQuery {
    atoms: BTreeSet<StyleAtom>,
    level_constraint: LevelConstraint,
}

impl StyleQuery {
    /// Creates a query from the given names, stereotypes, and level constraint.
    #[must_use]
    pub fn of_all(
        names: &[SName],
        stereotypes: &[String],
        level_constraint: LevelConstraint,
    ) -> Self {
        let mut atoms = BTreeSet::new();
        for name in names {
            atoms.insert(StyleAtom::of(*name));
        }
        for s in stereotypes {
            atoms.insert(StyleAtom::of_stereotype(s));
        }
        Self {
            atoms,
            level_constraint,
        }
    }

    /// Creates a query from the given names (no stereotypes).
    #[must_use]
    pub fn of(names: &[SName]) -> Self {
        Self::of_all(names, &[], LevelConstraint::none())
    }

    /// No tag, no depth constraint.
    #[must_use]
    pub fn empty() -> Self {
        Self::of(&[])
    }

    /// This same query, additionally requiring `stereotype`.
    #[must_use]
    pub fn with_stereotype(&self, stereotype: &str) -> Self {
        let mut atoms = self.atoms.clone();
        atoms.insert(StyleAtom::of_stereotype(stereotype));
        Self {
            atoms,
            level_constraint: self.level_constraint,
        }
    }

    /// This same query, additionally requiring `name`.
    #[must_use]
    pub fn add_sname(&self, name: SName) -> Self {
        let mut atoms = self.atoms.clone();
        atoms.insert(StyleAtom::of(name));
        Self {
            atoms,
            level_constraint: self.level_constraint,
        }
    }

    /// This same query, additionally constrained to depth `level`.
    #[must_use]
    pub fn add_level(&self, level: i32) -> Self {
        Self {
            atoms: self.atoms.clone(),
            level_constraint: LevelConstraint::of(level, self.level_constraint.is_star()),
        }
    }

    /// This same query, additionally starred.
    #[must_use]
    pub fn add_star(&self) -> Self {
        Self {
            atoms: self.atoms.clone(),
            level_constraint: LevelConstraint::of(
                self.level_constraint.get_level(),
                true,
            ),
        }
    }

    /// Returns the level constraint.
    #[must_use]
    pub fn get_level_constraint(&self) -> LevelConstraint {
        self.level_constraint
    }

    /// This query, unioned with `other`.
    #[must_use]
    pub fn merge_with(&self, other: &Self) -> Self {
        let mut atoms = self.atoms.clone();
        for atom in &other.atoms {
            atoms.insert(atom.clone());
        }
        let merged_level = self
            .level_constraint
            .get_level()
            .max(other.level_constraint.get_level());
        let merged_star = self.level_constraint.is_star() || other.level_constraint.is_star();
        Self {
            atoms,
            level_constraint: LevelConstraint::of(merged_level, merged_star),
        }
    }

    /// Returns the atoms in this query.
    #[must_use]
    pub fn get_atoms(&self) -> &BTreeSet<StyleAtom> {
        &self.atoms
    }


    /// Returns every stereotype atom this query carries.
    #[must_use]
    pub fn get_stereotypes(&self) -> Vec<String> {
        self.atoms
            .iter()
            .filter_map(|a| a.get_stereotype().map(String::from))
            .collect()
    }
}

impl std::fmt::Display for StyleQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let atoms: Vec<String> = self.atoms.iter().map(std::string::ToString::to_string).collect();
        write!(f, "{{{}}} {}", atoms.join(", "), self.level_constraint)
    }
}

impl std::fmt::Display for StyleAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => write!(f, "{:?}", n),
            Self::Stereotype(s) => write!(f, ".{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query() {
        let q = StyleQuery::empty();
        assert!(q.get_atoms().is_empty());
        assert!(!q.get_level_constraint().is_star());
    }

    #[test]
    fn with_stereotype() {
        let q = StyleQuery::of(&[SName::Root, SName::Document]);
        let q2 = q.with_stereotype("foo");
        assert_eq!(q2.get_stereotypes(), vec!["foo".to_string()]);
    }

    #[test]
    fn merge_with() {
        let q1 = StyleQuery::of(&[SName::Root]);
        let q2 = StyleQuery::of(&[SName::Document]);
        let merged = q1.merge_with(&q2);
        assert_eq!(merged.get_atoms().len(), 2);
    }
}
