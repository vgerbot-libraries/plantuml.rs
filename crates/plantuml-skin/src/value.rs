//! Value — style property value type.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/style/Value.java`
//! - `net/sourceforge/plantuml/style/ValueAbstract.java`
//! - `net/sourceforge/plantuml/style/ValueImpl.java`
//! - `net/sourceforge/plantuml/style/ValueNull.java`
//! - `net/sourceforge/plantuml/style/ValueColor.java`
//! - `net/sourceforge/plantuml/style/DarkString.java`

use plantuml_klimt::HColor;
use plantuml_klimt::HColorSet;
use plantuml_klimt::HColors;

use crate::specificity::Specificity;

/// Font face: weight (CSS 100-900) + italic axis.
///
/// Ported from: `net/sourceforge/plantuml/klimt/font/UFontFace.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UFontFace {
    pub weight: i32,
    pub italic: bool,
}

impl UFontFace {
    /// Normal weight (400), non-italic.
    #[must_use]
    pub fn normal() -> Self {
        Self {
            weight: 400,
            italic: false,
        }
    }

    /// Bold weight (700), non-italic.
    #[must_use]
    pub fn bold() -> Self {
        Self {
            weight: 700,
            italic: false,
        }
    }

    /// Normal weight (400), italic.
    #[must_use]
    pub fn italic() -> Self {
        Self {
            weight: 400,
            italic: true,
        }
    }

    /// Bold weight (700), italic.
    #[must_use]
    pub fn bold_italic() -> Self {
        Self {
            weight: 700,
            italic: true,
        }
    }

    /// Returns the CSS weight value.
    #[must_use]
    pub fn css_weight(self) -> i32 {
        self.weight
    }

    /// Creates a face with the given weight, preserving the italic setting.
    #[must_use]
    pub fn with_weight(self, weight: i32) -> Self {
        Self {
            weight,
            italic: self.italic,
        }
    }

    /// Parses a CSS weight keyword or numeric string.
    #[must_use]
    pub fn from_css_weight(s: &str) -> Option<Self> {
        match s {
            "bold" => Some(Self::bold()),
            "normal" | "plain" => Some(Self::normal()),
            "italic" => Some(Self::italic()),
            _ => {
                if let Ok(w) = s.parse::<i32>() {
                    if (100..=900).contains(&w) {
                        Some(Self {
                            weight: w,
                            italic: false,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

/// Horizontal alignment of text.
///
/// Ported from: `net/sourceforge/plantuml/klimt/geom/HorizontalAlignment.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

impl HorizontalAlignment {
    /// Parses an alignment from a string, case-insensitive.
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "left" => Some(Self::Left),
            "center" => Some(Self::Center),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}

impl std::fmt::Display for HorizontalAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Center => write!(f, "center"),
            Self::Right => write!(f, "right"),
        }
    }
}

/// A string value that may carry both a light and a dark half at a given
/// specificity.
///
/// Ported from: `net/sourceforge/plantuml/style/DarkString.java`
#[derive(Debug, Clone)]
pub struct DarkString {
    value1: Option<String>,
    value2: Option<String>,
    specificity: Specificity,
}

impl DarkString {
    /// Creates a new dark string.
    #[must_use]
    pub fn new(
        value1: Option<String>,
        value2: Option<String>,
        specificity: Specificity,
    ) -> Self {
        Self {
            value1,
            value2,
            specificity,
        }
    }

    /// Merges with `other`, keeping the higher-specificity side.
    #[must_use]
    pub fn merge_with(&self, other: &Self) -> Self {
        if (self.value2.is_none() && other.value2.is_none())
            || (self.value1.is_none() && other.value1.is_none())
        {
            if self.specificity.is_bigger_than(other.specificity) {
                return self.clone();
            }
            return other.clone();
        }
        if self.value2.is_none() && other.value1.is_none() {
            return Self::new(
                self.value1.clone(),
                other.value2.clone(),
                self.specificity,
            );
        }
        if other.value2.is_none() && self.value1.is_none() {
            return Self::new(
                other.value1.clone(),
                self.value2.clone(),
                other.specificity,
            );
        }
        if self.specificity.is_bigger_than(other.specificity) {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Returns this with the ancestor rank adjusted.
    #[must_use]
    pub fn with_ancestor_rank(&self, rank: i32) -> Self {
        Self::new(
            self.value1.clone(),
            self.value2.clone(),
            self.specificity.with_ancestor_rank(rank),
        )
    }

    /// Returns this with the stereotype count adjusted.
    #[must_use]
    pub fn with_stereotype_count(&self, count: i32) -> Self {
        Self::new(
            self.value1.clone(),
            self.value2.clone(),
            self.specificity.with_stereotype_count(count),
        )
    }

    /// Returns the primary (light) value.
    #[must_use]
    pub fn get_value1(&self) -> Option<&str> {
        self.value1.as_deref()
    }

    /// Returns the dark value.
    #[must_use]
    pub fn get_value2(&self) -> Option<&str> {
        self.value2.as_deref()
    }

    /// Returns the specificity.
    #[must_use]
    pub fn get_specificity(&self) -> Specificity {
        self.specificity
    }
}

/// A style property value.
///
/// In Java this is an interface (`Value`) with several implementations
/// (`ValueImpl`, `ValueNull`, `ValueColor`). In Rust we use an enum.
///
/// Ported from:
/// - `net/sourceforge/plantuml/style/Value.java`
/// - `net/sourceforge/plantuml/style/ValueImpl.java`
/// - `net/sourceforge/plantuml/style/ValueNull.java`
/// - `net/sourceforge/plantuml/style/ValueColor.java`
#[derive(Debug, Clone)]
pub enum Value {
    /// A string-based value (light/dark) with specificity.
    String(DarkString),
    /// A color value with specificity.
    Color {
        color: HColor,
        specificity: Specificity,
    },
    /// The null/absent value.
    Null,
}

impl Value {
    /// Creates a regular (light-only) string value at the given specificity.
    #[must_use]
    pub fn regular(value: &str, specificity: Specificity) -> Self {
        Self::String(DarkString::new(
            Some(value.to_string()),
            None,
            specificity,
        ))
    }

    /// Creates a dark-only string value at the given specificity.
    #[must_use]
    pub fn dark(value: &str, specificity: Specificity) -> Self {
        Self::String(DarkString::new(
            None,
            Some(value.to_string()),
            specificity,
        ))
    }

    /// Creates a color value.
    #[must_use]
    pub fn color(color: HColor, specificity: Specificity) -> Self {
        Self::Color { color, specificity }
    }

    /// The null value singleton.
    #[must_use]
    pub fn null() -> Self {
        Self::Null
    }

    /// Returns the value as a string.
    ///
    /// Ported from: `Value.asString()`.
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            Self::String(ds) => ds.get_value1().unwrap_or("").to_string(),
            Self::Color { color, .. } => format!("{color:?}"),
            Self::Null => String::new(),
        }
    }

    /// Returns the value as a color, resolved via the given color set.
    ///
    /// Ported from: `Value.asColor(HColorSet)`.
    #[must_use]
    pub fn as_color(&self, _set: &HColorSet) -> HColor {
        match self {
            Self::String(ds) => {
                let v = ds.get_value1().unwrap_or("");
                if v.eq_ignore_ascii_case("none") || v.eq_ignore_ascii_case("transparent") {
                    return HColors::transparent();
                }
                HColorSet::get_color_or_white(v)
            }
            Self::Color { color, .. } => color.clone(),
            Self::Null => HColors::black(),
        }
    }

    /// Returns the value as an integer (digits extracted from the string).
    ///
    /// Ported from: `Value.asInt()`.
    #[must_use]
    pub fn as_int(&self) -> i32 {
        self.extract_digits().parse().unwrap_or(0)
    }

    /// Returns the value as an integer, or -1 if no digits found.
    ///
    /// Ported from: `Value.asIntButMinusOneIfError()`.
    #[must_use]
    pub fn as_int_but_minus_one_if_error(&self) -> i32 {
        let s = self.extract_digits();
        if s.is_empty() {
            -1
        } else {
            s.parse().unwrap_or(-1)
        }
    }

    /// Returns the value as a double.
    ///
    /// Ported from: `Value.asDouble()`.
    #[must_use]
    pub fn as_double(&self) -> f64 {
        let s = self.as_string();
        let filtered: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
        if filtered.is_empty() {
            f64::NAN
        } else {
            filtered.parse().unwrap_or(f64::NAN)
        }
    }

    /// Returns the value as a double, or `default_value` if NaN.
    ///
    /// Ported from: `Value.asDoubleDefaultTo(double)`.
    #[must_use]
    pub fn as_double_default_to(&self, default_value: f64) -> f64 {
        let d = self.as_double();
        if d.is_nan() {
            default_value
        } else {
            d
        }
    }

    /// Returns the value as a boolean.
    ///
    /// Ported from: `Value.asBoolean()`.
    #[must_use]
    pub fn as_boolean(&self) -> bool {
        self.as_string().eq_ignore_ascii_case("true")
    }

    /// Returns the value as a font face.
    ///
    /// Ported from: `Value.asFontFace()`.
    #[must_use]
    pub fn as_font_face(&self) -> UFontFace {
        let raw = self.as_string();
        if raw.is_empty() {
            return UFontFace::normal();
        }
        let s = raw.trim().to_lowercase();
        match s.as_str() {
            "bold" => UFontFace::bold(),
            "italic" => UFontFace::italic(),
            "plain" | "normal" => UFontFace::normal(),
            _ => UFontFace::from_css_weight(&s).unwrap_or(UFontFace::normal()),
        }
    }

    /// Returns the value as a horizontal alignment.
    ///
    /// Ported from: `Value.asHorizontalAlignment()`.
    #[must_use]
    pub fn as_horizontal_alignment(&self) -> HorizontalAlignment {
        HorizontalAlignment::from_string(&self.as_string()).unwrap_or(HorizontalAlignment::Left)
    }

    /// Returns the specificity of this value.
    ///
    /// Ported from: `Value.getSpecificity()`.
    #[must_use]
    pub fn get_specificity(&self) -> Specificity {
        match self {
            Self::String(ds) => ds.get_specificity(),
            Self::Color { specificity, .. } => *specificity,
            Self::Null => Specificity::at_order(0),
        }
    }

    /// Merges this value with `other`, keeping the higher-specificity side.
    ///
    /// Ported from: `ValueImpl.mergeWith(Value)`.
    #[must_use]
    pub fn merge_with(&self, other: Option<&Value>) -> Value {
        let other = match other {
            None => return self.clone(),
            Some(o) => o,
        };
        match (self, other) {
            (Value::String(a), Value::String(b)) => {
                Value::String(a.merge_with(b))
            }
            (Value::String(_), Value::Color { specificity: os, .. }) => {
                if os.is_bigger_than(self.get_specificity()) {
                    other.clone()
                } else {
                    self.clone()
                }
            }
            (Value::Color { .. }, Value::Color { .. }) => {
                if other.get_specificity().is_bigger_than(self.get_specificity()) {
                    other.clone()
                } else {
                    self.clone()
                }
            }
            (Value::Null, _) => other.clone(),
            (_, Value::Null) => self.clone(),
            _ => self.clone(),
        }
    }

    /// Returns this value with the ancestor rank adjusted.
    ///
    /// Ported from: `ValueImpl.withAncestorRank(int)`.
    #[must_use]
    pub fn with_ancestor_rank(&self, rank: i32) -> Value {
        match self {
            Value::String(ds) => Value::String(ds.with_ancestor_rank(rank)),
            _ => self.clone(),
        }
    }

    fn extract_digits(&self) -> String {
        let s = self.as_string();
        s.chars()
            .filter(|c| c.is_ascii_digit())
            .collect()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.as_string() == other.as_string()
    }
}

impl Eq for Value {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_defaults() {
        let v = Value::null();
        assert_eq!(v.as_string(), "");
        assert_eq!(v.as_int(), 0);
        assert!(!v.as_boolean());
        assert_eq!(v.as_font_face(), UFontFace::normal());
    }

    #[test]
    fn string_value() {
        let v = Value::regular("42", Specificity::at_order(1));
        assert_eq!(v.as_string(), "42");
        assert_eq!(v.as_int(), 42);
        assert!((v.as_double() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn boolean_value() {
        let v = Value::regular("true", Specificity::at_order(1));
        assert!(v.as_boolean());
        let v = Value::regular("false", Specificity::at_order(1));
        assert!(!v.as_boolean());
    }

    #[test]
    fn font_face_parsing() {
        let v = Value::regular("bold", Specificity::at_order(1));
        assert_eq!(v.as_font_face(), UFontFace::bold());
        let v = Value::regular("italic", Specificity::at_order(1));
        assert_eq!(v.as_font_face(), UFontFace::italic());
        let v = Value::regular("700", Specificity::at_order(1));
        assert_eq!(v.as_font_face().css_weight(), 700);
    }

    #[test]
    fn horizontal_alignment() {
        let v = Value::regular("center", Specificity::at_order(1));
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Center);
    }

    #[test]
    fn as_double_default_to() {
        let v = Value::regular("abc", Specificity::at_order(1));
        assert!((v.as_double_default_to(5.0) - 5.0).abs() < 1e-9);
        let v = Value::regular("3.14", Specificity::at_order(1));
        assert!((v.as_double_default_to(5.0) - 3.14).abs() < 1e-9);
    }

    #[test]
    fn as_int_but_minus_one_if_error() {
        let v = Value::regular("42", Specificity::at_order(1));
        assert_eq!(v.as_int_but_minus_one_if_error(), 42);
        let v = Value::regular("abc", Specificity::at_order(1));
        assert_eq!(v.as_int_but_minus_one_if_error(), -1);
    }

    #[test]
    fn merge_with_higher_specificity() {
        let low = Value::regular("low", Specificity::at_order(1));
        let high = Value::regular("high", Specificity::at_order(2));
        let merged = low.merge_with(Some(&high));
        assert_eq!(merged.as_string(), "high");
    }
}
