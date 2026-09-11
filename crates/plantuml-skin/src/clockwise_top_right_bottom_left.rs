//! ClockwiseTopRightBottomLeft — four-sided padding/margin values.
//!
//! Ported from: `net/sourceforge/plantuml/style/ClockwiseTopRightBottomLeft.java`

/// Four-sided margin/padding specification (top, right, bottom, left).
///
/// Ported from: `net/sourceforge/plantuml/style/ClockwiseTopRightBottomLeft.java`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockwiseTopRightBottomLeft {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

impl ClockwiseTopRightBottomLeft {
    /// All four sides set to the same value.
    #[must_use]
    pub const fn same(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Zero on all sides.
    #[must_use]
    pub const fn none() -> Self {
        Self::same(0.0)
    }

    /// Creates a four-sided value.
    #[must_use]
    pub fn top_right_bottom_left(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Parses a CSS-style padding/margin string (1-4 space-separated integers).
    ///
    /// Ported from: `ClockwiseTopRightBottomLeft.read(String)`.
    #[must_use]
    pub fn read(value: &str) -> Self {
        if !is_only_numbers_and_spaces(value) {
            return Self::none();
        }
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => parts[0]
                .parse::<f64>()
                .map(Self::same)
                .unwrap_or(Self::none()),
            2 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                Self::top_right_bottom_left(a, b, a, b)
            }
            3 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                let c = parts[2].parse::<f64>().unwrap_or(0.0);
                Self::top_right_bottom_left(a, b, c, b)
            }
            4 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                let c = parts[2].parse::<f64>().unwrap_or(0.0);
                let d = parts[3].parse::<f64>().unwrap_or(0.0);
                Self::top_right_bottom_left(a, b, c, d)
            }
            _ => Self::none(),
        }
    }

    /// Creates a margin1/margin2 pair (top/bottom = margin1, left/right = margin2).
    #[must_use]
    pub fn margin1_margin2(margin1: f64, margin2: f64) -> Self {
        Self::top_right_bottom_left(margin1, margin2, margin1, margin2)
    }

    /// Returns a copy with `delta` added to the top.
    #[must_use]
    pub fn inc_top(self, delta: f64) -> Self {
        Self {
            top: self.top + delta,
            ..self
        }
    }

    #[must_use]
    pub fn get_top(self) -> f64 {
        self.top
    }

    #[must_use]
    pub fn get_right(self) -> f64 {
        self.right
    }

    #[must_use]
    pub fn get_bottom(self) -> f64 {
        self.bottom
    }

    #[must_use]
    pub fn get_left(self) -> f64 {
        self.left
    }

    /// Returns `true` if all sides are zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }
}

impl Default for ClockwiseTopRightBottomLeft {
    fn default() -> Self {
        Self::none()
    }
}

impl std::fmt::Display for ClockwiseTopRightBottomLeft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}:{}", self.top, self.right, self.bottom, self.left)
    }
}

fn is_only_numbers_and_spaces(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_value() {
        let p = ClockwiseTopRightBottomLeft::same(5.0);
        assert_eq!(p.get_top(), 5.0);
        assert_eq!(p.get_right(), 5.0);
        assert_eq!(p.get_bottom(), 5.0);
        assert_eq!(p.get_left(), 5.0);
    }

    #[test]
    fn read_single() {
        let p = ClockwiseTopRightBottomLeft::read("10");
        assert_eq!(p, ClockwiseTopRightBottomLeft::same(10.0));
    }

    #[test]
    fn read_two_values() {
        let p = ClockwiseTopRightBottomLeft::read("10 20");
        assert_eq!(p.get_top(), 10.0);
        assert_eq!(p.get_right(), 20.0);
        assert_eq!(p.get_bottom(), 10.0);
        assert_eq!(p.get_left(), 20.0);
    }

    #[test]
    fn read_four_values() {
        let p = ClockwiseTopRightBottomLeft::read("1 2 3 4");
        assert_eq!(p.get_top(), 1.0);
        assert_eq!(p.get_right(), 2.0);
        assert_eq!(p.get_bottom(), 3.0);
        assert_eq!(p.get_left(), 4.0);
    }

    #[test]
    fn read_non_numeric_returns_none() {
        let p = ClockwiseTopRightBottomLeft::read("abc");
        assert!(p.is_zero());
    }

    #[test]
    fn is_zero_check() {
        assert!(ClockwiseTopRightBottomLeft::none().is_zero());
        assert!(!ClockwiseTopRightBottomLeft::same(1.0).is_zero());
    }
}
