//! StyleBuilder — builds and resolves styles via a style index.
//!
//! Ported from: `net/sourceforge/plantuml/style/StyleBuilder.java`

use std::collections::HashMap;

use crate::merge_strategy::MergeStrategy;
use crate::style::Style;
use crate::style_query::StyleQuery;

/// Trait for automatic counter (used to assign specificity order values).
///
/// Ported from: `net/sourceforge/plantuml/style/AutomaticCounter.java`
pub trait AutomaticCounter {
    fn get_next_int(&mut self) -> i32;
}

/// Builds and resolves styles. Stores all loaded styles in a list and resolves
/// queries by iterating matching styles.
///
/// Ported from: `net/sourceforge/plantuml/style/StyleBuilder.java`
#[derive(Debug, Clone)]
pub struct StyleBuilder {
    styles: Vec<Style>,
    counter: i32,
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AutomaticCounter for StyleBuilder {
    fn get_next_int(&mut self) -> i32 {
        self.counter += 1;
        self.counter
    }
}

impl StyleBuilder {
    /// Creates an empty style builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            styles: Vec::new(),
            counter: 0,
        }
    }

    /// Creates a clone of this builder with the same styles and counter.
    ///
    /// Ported from: `StyleBuilder.cloneMe()`.
    #[must_use]
    pub fn clone_me(&self) -> Self {
        Self {
            styles: self.styles.clone(),
            counter: self.counter,
        }
    }

    /// Creates a style for a stereotype by exact-signature lookup.
    ///
    /// Ported from: `StyleBuilder.createStyleStereotype(String)`.
    #[must_use]
    pub fn create_style_stereotype(&self, name: &str) -> Style {
        let signature = StyleQuery::empty().with_stereotype(name);
        let mut result: Option<Style> = None;
        for style in &self.styles {
            if style.get_query() == &signature {
                result = Some(match result {
                    None => style.clone(),
                    Some(r) => r.merge_with(style, MergeStrategy::OverwriteExistingValue),
                });
            }
        }
        result.unwrap_or_else(|| Style::new(signature, HashMap::new()))
    }

    /// Returns a new builder with the given modified styles muted in.
    ///
    /// Ported from: `StyleBuilder.muteStyle(Collection)`.
    #[must_use]
    pub fn mute_style(&self, modified_styles: &[Style]) -> Self {
        let mut result = self.clone_me();
        for style in modified_styles {
            merge_or_append(&mut result.styles, style);
        }
        result
    }

    /// Loads a style internally (into this builder, mutating).
    ///
    /// Ported from: `StyleBuilder.loadInternal(StyleQuery, Style)`.
    pub fn load_internal(&mut self, new_style: Style) {
        merge_or_append(&mut self.styles, &new_style);
    }

    /// Returns the merged style for the given query, or an empty style if
    /// nothing matches.
    ///
    /// Ported from: `StyleBuilder.getMergedStyle(StyleQuery)`.
    #[must_use]
    pub fn get_merged_style(&self, query: &StyleQuery) -> Style {
        let mut merged: Option<Style> = None;
        for style in &self.styles {
            if style_matches(style.get_query(), query) {
                merged = Some(match merged {
                    None => style.clone(),
                    Some(m) => m.merge_with(style, MergeStrategy::OverwriteExistingValue),
                });
            }
        }
        merged.unwrap_or_else(|| Style::new(query.clone(), HashMap::new()))
    }

    /// Returns the merged style for the given query with ancestor rank applied
    /// to starred declarations.
    ///
    /// Ported from: `StyleBuilder.getMergedStyleSpecial(StyleQuery, int)`.
    #[must_use]
    pub fn get_merged_style_special(&self, query: &StyleQuery, ancestor_rank: i32) -> Style {
        let mut merged: Option<Style> = None;
        for style in &self.styles {
            if style_matches(style.get_query(), query) {
                let tmp = if style.get_query().get_level_constraint().is_star() {
                    style.with_ancestor_rank(ancestor_rank)
                } else {
                    style.clone()
                };
                merged = Some(match merged {
                    None => tmp,
                    Some(m) => m.merge_with(&tmp, MergeStrategy::OverwriteExistingValue),
                });
            }
        }
        merged.unwrap_or_else(|| Style::new(query.clone(), HashMap::new()))
    }

    /// Returns all loaded styles.
    #[must_use]
    pub fn get_all_styles(&self) -> &[Style] {
        &self.styles
    }
}

/// Folds `new_style` into whichever element of `list` already carries the
/// same signature, or appends it as a new entry.
fn merge_or_append(list: &mut Vec<Style>, new_style: &Style) {
    let signature = new_style.get_query();
    for (i, s) in list.iter().enumerate() {
        if s.get_query() == signature {
            let merged = s.merge_with(new_style, MergeStrategy::OverwriteExistingValue);
            list[i] = merged;
            return;
        }
    }
    list.push(new_style.clone());
}

/// Checks whether a declaration query matches a lookup query.
///
/// A declaration matches if every atom in the declaration is also in the query
/// (subset check) and the level constraints are compatible.
fn style_matches(declaration: &StyleQuery, query: &StyleQuery) -> bool {
    let decl_atoms = declaration.get_atoms();
    let query_atoms = query.get_atoms();

    // Every atom in the declaration must also be in the query (subset check).
    for atom in decl_atoms {
        if !query_atoms.contains(atom) {
            return false;
        }
    }

    // Level constraint check.
    let decl_lc = declaration.get_level_constraint();
    let query_lc = query.get_level_constraint();
    crate::level_constraint::LevelConstraint::matches(decl_lc, query_lc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p_name::PName;
    use crate::s_name::SName;
    use crate::specificity::Specificity;
    use crate::value::Value;

    #[test]
    fn empty_builder() {
        let sb = StyleBuilder::new();
        let q = StyleQuery::of(&[SName::Root]);
        let s = sb.get_merged_style(&q);
        assert!(!s.has_value(PName::FontName));
    }

    #[test]
    fn load_and_query() {
        let mut sb = StyleBuilder::new();
        let q = StyleQuery::of(&[SName::Root, SName::Document]);
        let mut map = HashMap::new();
        let spec = Specificity::at_order(sb.clone().get_next_int());
        map.insert(PName::FontName, Value::regular("Arial", spec));
        sb.load_internal(Style::new(q.clone(), map));

        let s = sb.get_merged_style(&q);
        assert_eq!(s.value(PName::FontName).as_string(), "Arial");
    }

    #[test]
    fn mute_style() {
        let mut sb = StyleBuilder::new();
        let q = StyleQuery::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(
            PName::FontName,
            Value::regular("Arial", Specificity::at_order(1)),
        );
        sb.load_internal(Style::new(q.clone(), map));

        let mut override_map = HashMap::new();
        override_map.insert(
            PName::FontName,
            Value::regular("Helvetica", Specificity::at_order(2)),
        );
        let override_style = Style::new(q, override_map);
        let sb2 = sb.mute_style(&[override_style]);

        let s = sb2.get_merged_style(&StyleQuery::of(&[SName::Root]));
        assert_eq!(s.value(PName::FontName).as_string(), "Helvetica");
    }

    #[test]
    fn subset_matching() {
        let mut sb = StyleBuilder::new();
        let q = StyleQuery::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(
            PName::FontSize,
            Value::regular("16", Specificity::at_order(1)),
        );
        sb.load_internal(Style::new(q, map));

        // Query with more atoms should still match the Root-only declaration.
        let s = sb.get_merged_style(&StyleQuery::of(&[SName::Root, SName::Document]));
        assert_eq!(s.value(PName::FontSize).as_string(), "16");
    }
}
