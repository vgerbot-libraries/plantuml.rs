//! `UPath` — path shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/UPath.java`


/// A path segment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum USegmentType {
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Close,
}

/// A path segment.
#[derive(Debug, Clone)]
pub struct USegment {
    segment_type: USegmentType,
    coords: Vec<f64>,
}

impl USegment {
    #[must_use]
    pub const fn new(segment_type: USegmentType, coords: Vec<f64>) -> Self {
        Self { segment_type, coords }
    }

    #[must_use]
    pub const fn segment_type(&self) -> USegmentType {
        self.segment_type
    }

    #[must_use]
    pub fn coords(&self) -> &[f64] {
        &self.coords
    }
}

/// A path shape composed of segments.
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/UPath.java`
#[derive(Debug, Clone)]
pub struct UPath {
    segments: Vec<USegment>,
    x: f64,
    y: f64,
}

impl UPath {
    /// Creates a new empty `UPath` at (0, 0).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    /// Creates a new `UPath` at the given position.
    #[must_use]
    pub const fn at(x: f64, y: f64) -> Self {
        Self {
            segments: Vec::new(),
            x,
            y,
        }
    }

    /// Adds a move-to segment.
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.segments
            .push(USegment::new(USegmentType::MoveTo, vec![x, y]));
    }

    /// Adds a line-to segment.
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.segments
            .push(USegment::new(USegmentType::LineTo, vec![x, y]));
    }

    /// Adds a cubic bezier segment.
    pub fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.segments
            .push(USegment::new(USegmentType::CubicTo, vec![x1, y1, x2, y2, x3, y3]));
    }

    /// Closes the path.
    pub fn close(&mut self) {
        self.segments
            .push(USegment::new(USegmentType::Close, Vec::new()));
    }

    /// Returns the segments.
    #[must_use]
    pub fn segments(&self) -> &[USegment] {
        &self.segments
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    /// Returns the number of segments.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` if the path has no segments.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl Default for UPath {
    fn default() -> Self {
        Self::new()
    }
}

crate::impl_ushape!(UPath);
