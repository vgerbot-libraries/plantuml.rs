//! Style — a collection of PName→Value mappings with a selector query.
//!
//! Ported from: `net/sourceforge/plantuml/style/Style.java`

use std::collections::HashMap;

use plantuml_klimt::HColor;
use plantuml_klimt::HColorSet;
use plantuml_klimt::HColors;

use crate::clockwise_top_right_bottom_left::ClockwiseTopRightBottomLeft;
use crate::merge_strategy::MergeStrategy;
use crate::p_name::PName;
use crate::specificity::Specificity;
use crate::style_query::StyleQuery;
use crate::value::{HorizontalAlignment, UFontFace, Value};

/// A style: a selector query plus a set of property→value mappings.
///
/// Ported from: `net/sourceforge/plantuml/style/Style.java`
#[derive(Debug, Clone)]
pub struct Style {
    map: HashMap<PName, Value>,
    query: StyleQuery,
}

/// Wildcard sentinel used in stereotype-only style names.
pub const STAR: &str = "*";

/// Style ID constants for bordered text blocks.
pub const ID_TITLE: &str = "_title";
pub const ID_CAPTION: &str = "_caption";
pub const ID_LEGEND: &str = "_legend";

impl Style {
    /// Creates a new style with the given query and property map.
    #[must_use]
    pub fn new(query: StyleQuery, map: HashMap<PName, Value>) -> Self {
        Self { map, query }
    }

    /// Returns a copy with the ancestor rank applied to all values.
    ///
    /// Ported from: `Style.withAncestorRank(int)`.
    #[must_use]
    pub fn with_ancestor_rank(&self, rank: i32) -> Self {
        let map = self
            .map
            .iter()
            .map(|(k, v)| (*k, v.with_ancestor_rank(rank)))
            .collect();
        Self {
            map,
            query: self.query.clone(),
        }
    }

    /// Returns the value for `name`, or `Value::Null` if not set.
    ///
    /// Ported from: `Style.value(PName)`.
    #[must_use]
    pub fn value(&self, name: PName) -> &Value {
        self.map
            .get(&name)
            .map_or(&Value::Null, |v| v)
    }

    /// Returns `true` if this style has a value for `name`.
    ///
    /// Ported from: `Style.hasValue(PName)`.
    #[must_use]
    pub fn has_value(&self, name: PName) -> bool {
        self.map.contains_key(&name)
    }

    /// Returns the shadowing value (defaults to 1.5 if not set, 0 if null).
    ///
    /// Ported from: `Style.getShadowing()`.
    #[must_use]
    pub fn get_shadowing(&self) -> f64 {
        match self.map.get(&PName::Shadowing) {
            None => 0.0,
            Some(v) => v.as_double_default_to(1.5),
        }
    }

    /// Merges this style with `other` using the given strategy.
    ///
    /// Ported from: `Style.mergeWith(Style, MergeStrategy)`.
    #[must_use]
    pub fn merge_with(&self, other: &Style, strategy: MergeStrategy) -> Self {
        let mut both = self.map.clone();
        for (key, val) in &other.map {
            let previous = self.map.get(key);
            if let Some(prev) = previous {
                if prev.get_specificity().has_stereotype()
                    && strategy == MergeStrategy::KeepExistingValueOfStereotype
                {
                    continue;
                }
            }
            let merged = val.merge_with(previous);
            both.insert(*key, merged);
        }
        Self {
            map: both,
            query: self.query.merge_with(&other.query),
        }
    }

    /// Overrides a single property with a color.
    ///
    /// Ported from: `Style.eventuallyOverride(PName, HColor)`.
    #[must_use]
    pub fn eventually_override_color(&self, param: PName, color: HColor) -> Self {
        let old_specificity = self
            .map
            .get(&param)
            .map_or(Specificity::at_order(0), Value::get_specificity);
        let mut result = self.map.clone();
        result.insert(param, Value::color(color, old_specificity));
        Self {
            map: result,
            query: self.query.clone(),
        }
    }

    /// Overrides a single property with a double value.
    #[must_use]
    pub fn eventually_override_double(&self, param: PName, value: f64) -> Self {
        self.eventually_override_string(param, &value.to_string())
    }

    /// Overrides a single property with a string value (forced override).
    ///
    /// Ported from: `Style.eventuallyOverride(PName, String)`.
    #[must_use]
    pub fn eventually_override_string(&self, param: PName, value: &str) -> Self {
        let mut result = self.map.clone();
        result.insert(
            param,
            Value::regular(value, Specificity::forced_override()),
        );
        Self {
            map: result,
            query: self.query.clone(),
        }
    }

    /// Returns the query/selector of this style.
    ///
    /// Ported from: `Style.getQuery()`.
    #[must_use]
    pub fn get_query(&self) -> &StyleQuery {
        &self.query
    }

    /// Builds a `UFont` from the style properties FontName, FontStyle,
    /// FontWeight, and FontSize.
    ///
    /// Ported from: `Style.getUFont()`.
    #[must_use]
    pub fn get_font_family(&self) -> String {
        self.value(PName::FontName).as_string()
    }

    /// Returns the font size from the style (defaults to 14 if invalid).
    #[must_use]
    pub fn get_font_size(&self) -> i32 {
        let size = self.value(PName::FontSize).as_int_but_minus_one_if_error();
        if size == -1 {
            14
        } else {
            size
        }
    }

    /// Returns the font face (style + weight) from the style.
    #[must_use]
    pub fn get_font_face(&self) -> UFontFace {
        let mut face = self.value(PName::FontStyle).as_font_face();
        let weight_face = self.value(PName::FontWeight).as_font_face();
        if weight_face.css_weight() != 400 {
            face = face.with_weight(weight_face.css_weight());
        }
        face
    }

    /// Returns the padding as a `ClockwiseTopRightBottomLeft`.
    ///
    /// Ported from: `Style.getPadding()`.
    #[must_use]
    pub fn get_padding(&self) -> ClockwiseTopRightBottomLeft {
        let padding = self.value(PName::Padding).as_string();
        ClockwiseTopRightBottomLeft::read(&padding)
    }

    /// Returns the margin as a `ClockwiseTopRightBottomLeft`.
    ///
    /// Ported from: `Style.getMargin()`.
    #[must_use]
    pub fn get_margin(&self) -> ClockwiseTopRightBottomLeft {
        let margin = self.value(PName::Margin).as_string();
        ClockwiseTopRightBottomLeft::read(&margin)
    }

    /// Returns the horizontal alignment from the style.
    ///
    /// Ported from: `Style.getHorizontalAlignment()`.
    #[must_use]
    pub fn get_horizontal_alignment(&self) -> HorizontalAlignment {
        self.value(PName::HorizontalAlignment)
            .as_horizontal_alignment()
    }

    /// Returns the round corner value.
    #[must_use]
    pub fn get_round_corner(&self) -> f64 {
        self.value(PName::RoundCorner).as_double()
    }

    /// Returns the line thickness as a double.
    #[must_use]
    pub fn get_line_thickness(&self) -> f64 {
        self.value(PName::LineThickness).as_double()
    }

    /// Returns the background color, resolved via the given color set.
    #[must_use]
    pub fn get_background_color(&self, set: &HColorSet) -> HColor {
        self.value(PName::BackGroundColor).as_color(set)
    }

    /// Returns the line color, resolved via the given color set.
    #[must_use]
    pub fn get_line_color(&self, set: &HColorSet) -> HColor {
        self.value(PName::LineColor).as_color(set)
    }

    /// Returns the font color, resolved via the given color set.
    #[must_use]
    pub fn get_font_color(&self, set: &HColorSet) -> HColor {
        self.value(PName::FontColor).as_color(set)
    }

    /// Applies the line color and stroke to a color, returning the color
    /// (or transparent if no line color is set).
    ///
    /// Ported from: `Style.applyStrokeAndLineColor(UGraphic, HColorSet)`.
    #[must_use]
    pub fn get_stroke_line_color(&self, color_set: &HColorSet) -> HColor {
        let color = self.value(PName::LineColor).as_color(color_set);
        if color == HColors::black() && !self.has_value(PName::LineColor) {
            HColors::transparent()
        } else {
            color
        }
    }

    /// Returns the property map.
    #[must_use]
    pub fn get_map(&self) -> &HashMap<PName, Value> {
        &self.map
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.query, self.map.keys().collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::s_name::SName;

    #[test]
    fn empty_style() {
        let q = StyleQuery::of(&[SName::Root]);
        let s = Style::new(q.clone(), HashMap::new());
        assert!(!s.has_value(PName::FontName));
        assert_eq!(s.value(PName::FontName).as_string(), "");
    }

    #[test]
    fn get_shadowing_default() {
        let q = StyleQuery::of(&[SName::Root]);
        let s = Style::new(q, HashMap::new());
        assert_eq!(s.get_shadowing(), 0.0);
    }

    #[test]
    fn eventually_override() {
        let q = StyleQuery::of(&[SName::Root]);
        let s = Style::new(q, HashMap::new());
        let s2 = s.eventually_override_string(PName::FontName, "Arial");
        assert_eq!(s2.value(PName::FontName).as_string(), "Arial");
        assert!(!s.has_value(PName::FontName));
    }

    #[test]
    fn merge_with_overwrite() {
        let q = StyleQuery::of(&[SName::Root]);
        let mut map1 = HashMap::new();
        map1.insert(
            PName::FontName,
            Value::regular("Arial", Specificity::at_order(1)),
        );
        let s1 = Style::new(q.clone(), map1);

        let mut map2 = HashMap::new();
        map2.insert(
            PName::FontSize,
            Value::regular("16", Specificity::at_order(2)),
        );
        let s2 = Style::new(q, map2);

        let merged = s1.merge_with(&s2, MergeStrategy::OverwriteExistingValue);
        assert_eq!(merged.value(PName::FontName).as_string(), "Arial");
        assert_eq!(merged.value(PName::FontSize).as_string(), "16");
    }

    #[test]
    fn get_font_size_default() {
        let q = StyleQuery::of(&[SName::Root]);
        let s = Style::new(q, HashMap::new());
        assert_eq!(s.get_font_size(), 14);
    }
}
