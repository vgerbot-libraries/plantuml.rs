//! `DisplayPositioned` — a display with alignment and position info.
//!
//! Ported from: `net/sourceforge/plantuml/abel/DisplayPositioned.java`
use plantuml_klimt::Display;
use plantuml_klimt::{HorizontalAlignment, VerticalAlignment};

/// A `Display` with horizontal/vertical alignment and optional line location.
///
/// Used for title, caption, legend, header, footer in `TitledDiagram`.
///
/// Ported from: `net/sourceforge/plantuml/abel/DisplayPositioned.java`
#[derive(Debug, Clone, Default)]
pub struct DisplayPositioned {
    display: Display,
    horizontal_alignment: Option<HorizontalAlignment>,
    vertical_alignment: Option<VerticalAlignment>,
}

impl DisplayPositioned {
    /// Creates a positioned display with the given display and alignments.
    ///
    /// Ported from: `DisplayPositioned.single(Display, HA, VA)`.
    #[must_use]
    pub const fn single(
        display: Display,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        Self {
            display,
            horizontal_alignment: Some(horizontal_alignment),
            vertical_alignment: Some(vertical_alignment),
        }
    }

    /// Creates a "none" positioned display (null display with alignments).
    ///
    /// Ported from: `DisplayPositioned.none(HA, VA)`.
    #[must_use]
    pub const fn none(
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        Self {
            display: Display::NULL,
            horizontal_alignment: Some(horizontal_alignment),
            vertical_alignment: Some(vertical_alignment),
        }
    }

    /// Returns a new `DisplayPositioned` with the display replaced.
    ///
    /// Ported from: `DisplayPositioned.withDisplay(Display)`.
    #[must_use]
    pub const fn with_display(&self, display: Display) -> Self {
        Self {
            display,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
        }
    }

    /// Returns a new `DisplayPositioned` with the horizontal alignment replaced.
    ///
    /// Ported from: `DisplayPositioned.withHorizontalAlignment(HA)`.
    #[must_use]
    pub fn with_horizontal_alignment(&self, ha: HorizontalAlignment) -> Self {
        Self {
            display: self.display.clone(),
            horizontal_alignment: Some(ha),
            vertical_alignment: self.vertical_alignment,
        }
    }

    /// Returns `true` if this is the null display.
    ///
    /// Ported from: `DisplayPositioned.isNull()`.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.display.is_null()
    }

    /// Returns the display.
    ///
    /// Ported from: `DisplayPositioned.getDisplay()`.
    #[must_use]
    pub const fn get_display(&self) -> &Display {
        &self.display
    }

    /// Returns the horizontal alignment, if set.
    ///
    /// Ported from: `DisplayPositioned.getHorizontalAlignment()`.
    #[must_use]
    pub const fn get_horizontal_alignment(&self) -> Option<HorizontalAlignment> {
        self.horizontal_alignment
    }

    /// Returns the vertical alignment, if set.
    ///
    /// Ported from: `DisplayPositioned.getVerticalAlignment()`.
    #[must_use]
    pub const fn get_vertical_alignment(&self) -> Option<VerticalAlignment> {
        self.vertical_alignment
    }
}
