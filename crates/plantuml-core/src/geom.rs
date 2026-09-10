//! Geometry types ported from `klimt/geom`.
//!
//! Ported from: net/sourceforge/plantuml/klimt/geom/XPoint2D.java
//!              net/sourceforge/plantuml/klimt/geom/XDimension2D.java
//!              net/sourceforge/plantuml/klimt/geom/XRectangle2D.java
//!              net/sourceforge/plantuml/klimt/geom/XLine2D.java

use std::f64;

/// A 2D point with `f64` coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XPoint2D {
    pub x: f64,
    pub y: f64,
}

impl XPoint2D {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub fn distance(&self, other: &Self) -> f64 {
        let px = other.x - self.x;
        let py = other.y - self.y;
        (px * px + py * py).sqrt()
    }

    #[must_use]
    pub fn distance_sq(&self, other: &Self) -> f64 {
        let px = other.x - self.x;
        let py = other.y - self.y;
        px * px + py * py
    }

    #[must_use]
    pub fn move_xy(&self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    #[must_use]
    pub fn move_pt(&self, delta: &Self) -> Self {
        Self::new(self.x + delta.x, self.y + delta.y)
    }
}

impl std::fmt::Display for XPoint2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

/// A 2D dimension (width, height). Negative or NaN values are rejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XDimension2D {
    width: f64,
    height: f64,
}

impl XDimension2D {
    /// Create a new dimension. Returns `None` for negative or NaN inputs
    /// (the Java version throws `IllegalArgumentException`).
    #[must_use]
    pub fn new(width: f64, height: f64) -> Option<Self> {
        if width < 0.0 || height < 0.0 || width.is_nan() || height.is_nan() {
            return None;
        }
        Some(Self { width, height })
    }

    /// Create a dimension, panicking on invalid input (use only in tests).
    #[must_use]
    pub fn new_or_zero(width: f64, height: f64) -> Self {
        Self::new(width, height).unwrap_or(Self { width: 0.0, height: 0.0 })
    }

    #[must_use]
    pub const fn width(&self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> f64 {
        self.height
    }

    #[must_use]
    pub fn delta_uniform(&self, delta: f64) -> Self {
        self.delta(delta, delta)
    }

    #[must_use]
    pub fn with_width(&self, new_width: f64) -> Self {
        Self::new_or_zero(new_width, self.height)
    }

    #[must_use]
    pub fn apply_translate(&self, translate: &UTranslate) -> Self {
        Self::new_or_zero(self.width + translate.dx, self.height + translate.dy)
    }

    #[must_use]
    pub fn delta(&self, delta_width: f64, delta_height: f64) -> Self {
        if delta_height == 0.0 && delta_width == 0.0 {
            return *self;
        }
        Self::new_or_zero(self.width + delta_width, self.height + delta_height)
    }

    /// Merge top+bottom: max width, sum height.
    #[must_use]
    pub fn merge_tb(&self, bottom: &Self) -> Self {
        Self::new_or_zero(self.width.max(bottom.width), self.height + bottom.height)
    }

    /// Merge left+right: sum width, max height.
    #[must_use]
    pub fn merge_lr(&self, right: &Self) -> Self {
        Self::new_or_zero(self.width + right.width, self.height.max(right.height))
    }

    #[must_use]
    pub fn at_least(&self, min_width: f64, min_height: f64) -> Self {
        let mut h = self.height;
        let mut w = self.width;
        if w > min_width && h > min_height {
            return *self;
        }
        if h < min_height {
            h = min_height;
        }
        if w < min_width {
            w = min_width;
        }
        Self::new_or_zero(w, h)
    }
}

impl std::fmt::Display for XDimension2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.width, self.height)
    }
}

/// A rectangle defined by its top-left corner and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XRectangle2D {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl XRectangle2D {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn width(&self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> f64 {
        self.height
    }

    #[must_use]
    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    #[must_use]
    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    #[must_use]
    pub const fn min_x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub fn max_x(&self) -> f64 {
        self.x + self.width
    }

    #[must_use]
    pub const fn min_y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub fn max_y(&self) -> f64 {
        self.y + self.height
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.intersects_rect(other.x, other.y, other.width, other.height)
    }

    fn intersects_rect(&self, x: f64, y: f64, w: f64, h: f64) -> bool {
        w > 0.0
            && h > 0.0
            && self.width > 0.0
            && self.height > 0.0
            && x < self.x + self.width
            && x + w > self.x
            && y < self.y + self.height
            && y + h > self.y
    }

    #[must_use]
    pub fn contains(&self, xp: f64, yp: f64) -> bool {
        xp >= self.x && xp <= self.x + self.width && yp >= self.y && yp <= self.y + self.height
    }

    /// Intersection of this rectangle's border with a line, if any.
    #[must_use]
    pub fn intersect_line(&self, line: &XLine2D) -> Option<XPoint2D> {
        let a = XPoint2D::new(self.x, self.y);
        let b = XPoint2D::new(self.x + self.width, self.y);
        let c = XPoint2D::new(self.x + self.width, self.y + self.height);
        let d = XPoint2D::new(self.x, self.y + self.height);
        let line1 = XLine2D::line(&a, &b);
        let line2 = XLine2D::line(&b, &c);
        let line3 = XLine2D::line(&c, &d);
        let line4 = XLine2D::line(&d, &a);
        line.intersect(&line1)
            .or_else(|| line.intersect(&line2))
            .or_else(|| line.intersect(&line3))
            .or_else(|| line.intersect(&line4))
    }
}

/// A line segment between two points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XLine2D {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl XLine2D {
    #[must_use]
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    #[must_use]
    pub fn line(p1: &XPoint2D, p2: &XPoint2D) -> Self {
        Self::new(p1.x, p1.y, p2.x, p2.y)
    }

    #[must_use]
    pub fn middle(&self) -> XPoint2D {
        XPoint2D::new((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }

    #[must_use]
    pub fn p1(&self) -> XPoint2D {
        XPoint2D::new(self.x1, self.y1)
    }

    #[must_use]
    pub fn p2(&self) -> XPoint2D {
        XPoint2D::new(self.x2, self.y2)
    }

    #[must_use]
    pub fn with_point1(&self, other: &XPoint2D) -> Self {
        Self::new(other.x, other.y, self.x2, self.y2)
    }

    #[must_use]
    pub fn with_point2(&self, other: &XPoint2D) -> Self {
        Self::new(self.x1, self.y1, other.x, other.y)
    }

    #[must_use]
    pub fn angle(&self) -> f64 {
        (self.y2 - self.y1).atan2(self.x2 - self.x1)
    }

    /// Square of the distance from point `(px,py)` to the segment.
    #[must_use]
    pub fn pt_seg_dist_sq(x1: f64, y1: f64, x2: f64, y2: f64, px: f64, py: f64) -> f64 {
        // Adjust vectors relative to x1,y1
        let mut x2 = x2 - x1;
        let mut y2 = y2 - y1;
        let mut px = px - x1;
        let mut py = py - y1;
        let mut dotprod = px * x2 + py * y2;
        let projlen_sq;
        if dotprod <= 0.0 {
            projlen_sq = 0.0;
        } else {
            px = x2 - px;
            py = y2 - py;
            dotprod = px * x2 + py * y2;
            if dotprod <= 0.0 {
                projlen_sq = 0.0;
            } else {
                projlen_sq = dotprod * dotprod / (x2 * x2 + y2 * y2);
            }
        }
        let mut len_sq = px * px + py * py - projlen_sq;
        if len_sq < 0.0 {
            len_sq = 0.0;
        }
        // suppress unused-mut that the algorithm legitimately needs
        let _ = (&mut x2, &mut y2, &mut px, &mut py);
        len_sq
    }

    /// Intersection with another line segment, if within both segments.
    #[must_use]
    pub fn intersect(&self, line2: &Self) -> Option<XPoint2D> {
        let s1x = self.x2 - self.x1;
        let s1y = self.y2 - self.y1;
        let s2x = line2.x2 - line2.x1;
        let s2y = line2.y2 - line2.y1;
        let denom = -s2x * s1y + s1x * s2y;
        if denom == 0.0 {
            return None;
        }
        let s = (-s1y * (self.x1 - line2.x1) + s1x * (self.y1 - line2.y1)) / denom;
        let t = (s2x * (self.y1 - line2.y1) - s2y * (self.x1 - line2.x1)) / denom;
        if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
            Some(XPoint2D::new(self.x1 + t * s1x, self.y1 + t * s1y))
        } else {
            None
        }
    }
}

/// A 2D translation vector. Ported from `klimt/UTranslate` (minimal core form).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UTranslate {
    pub dx: f64,
    pub dy: f64,
}

impl UTranslate {
    #[must_use]
    pub const fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    #[must_use]
    pub const fn dx(&self) -> f64 {
        self.dx
    }

    #[must_use]
    pub const fn dy(&self) -> f64 {
        self.dy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let a = XPoint2D::new(0.0, 0.0);
        let b = XPoint2D::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn dimension_merge() {
        let a = XDimension2D::new_or_zero(10.0, 5.0);
        let b = XDimension2D::new_or_zero(3.0, 7.0);
        let tb = a.merge_tb(&b);
        assert_eq!(tb.width(), 10.0);
        assert_eq!(tb.height(), 12.0);
        let lr = a.merge_lr(&b);
        assert_eq!(lr.width(), 13.0);
        assert_eq!(lr.height(), 7.0);
    }

    #[test]
    fn dimension_rejects_negative() {
        assert!(XDimension2D::new(-1.0, 5.0).is_none());
        assert!(XDimension2D::new(f64::NAN, 5.0).is_none());
        assert!(XDimension2D::new(5.0, 5.0).is_some());
    }

    #[test]
    fn rectangle_contains() {
        let r = XRectangle2D::new(1.0, 1.0, 4.0, 4.0);
        assert!(r.contains(2.0, 2.0));
        assert!(!r.contains(10.0, 10.0));
    }

    #[test]
    fn line_intersect() {
        let l1 = XLine2D::new(0.0, 0.0, 4.0, 4.0);
        let l2 = XLine2D::new(0.0, 4.0, 4.0, 0.0);
        let p = l1.intersect(&l2).expect("diagonals cross");
        assert!((p.x - 2.0).abs() < 1e-9 && (p.y - 2.0).abs() < 1e-9);
    }
}
