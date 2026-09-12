//! `TitledDiagram` — shared base struct for all non-sequence diagram types.
//!
//! Ported from: `net/sourceforge/plantuml/TitledDiagram.java`
//!
//! In Java, `TitledDiagram` is an abstract class extended by all diagram
//! types except sequence. In Rust, we use composition: concrete diagram
//! types embed a `TitledDiagram` field and delegate to it.

use std::sync::atomic::{AtomicBool, Ordering};

use plantuml_core::DiagramType;
use plantuml_klimt::Display;
use plantuml_model::DisplayPositioned;
use plantuml_skin::{HorizontalAlignment, Pragma, SkinParam, VerticalAlignment};

/// Force Smetana layout for all diagrams (for testing).
/// Matches Java's `TitledDiagram.FORCE_SMETANA`.
pub static FORCE_SMETANA: AtomicBool = AtomicBool::new(false);

/// Force ELK layout for all diagrams (for testing).
/// Matches Java's `TitledDiagram.FORCE_ELK`.
pub static FORCE_ELK: AtomicBool = AtomicBool::new(false);

/// Shared base state for all titled diagram types.
///
/// Holds the title, caption, legend, header, footer, main frame,
/// skin parameters, pragma, and layout flags. Concrete diagram types
/// embed this struct and call its methods for common operations.
///
/// Ported from: `net/sourceforge/plantuml/TitledDiagram.java`
#[derive(Debug, Clone)]
pub struct TitledDiagram {
    diagram_type: DiagramType,
    title: DisplayPositioned,
    caption: DisplayPositioned,
    legend: DisplayPositioned,
    header: DisplayPositioned,
    footer: DisplayPositioned,
    main_frame: DisplayPositioned,
    skin_param: SkinParam,
    pragma: Pragma,
    namespace_separator: Option<String>,
    use_smetana: bool,
    use_elk: bool,
    skin_param_used: bool,
}

impl TitledDiagram {
    /// Creates a new `TitledDiagram` for the given diagram type.
    ///
    /// Ported from: `TitledDiagram(UmlSource, DiagramType, Previous, PreprocessingArtifact)`.
    #[must_use]
    pub fn new(diagram_type: DiagramType) -> Self {
        Self {
            diagram_type,
            title: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Top,
            ),
            caption: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Bottom,
            ),
            legend: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Bottom,
            ),
            header: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Top,
            ),
            footer: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Bottom,
            ),
            main_frame: DisplayPositioned::none(
                HorizontalAlignment::Center,
                VerticalAlignment::Center,
            ),
            skin_param: SkinParam::default(),
            pragma: Pragma::default(),
            namespace_separator: None,
            use_smetana: false,
            use_elk: false,
            skin_param_used: false,
        }
    }

    // --- Title ---

    /// Sets the diagram title. Ignored if null or all-whitespace.
    ///
    /// Ported from: `TitledDiagram.setTitle(DisplayPositioned)`.
    pub fn set_title(&mut self, title: DisplayPositioned) {
        if title.is_null() || title.get_display().is_white() {
            return;
        }
        self.title = title;
    }

    /// Returns the diagram title.
    #[must_use]
    pub fn get_title(&self) -> &DisplayPositioned {
        &self.title
    }

    /// Returns the title as a `Display`.
    #[must_use]
    pub fn get_title_display(&self) -> &Display {
        self.title.get_display()
    }

    // --- Caption ---

    pub fn set_caption(&mut self, caption: DisplayPositioned) {
        self.caption = caption;
    }

    #[must_use]
    pub fn get_caption(&self) -> &DisplayPositioned {
        &self.caption
    }

    // --- Legend ---

    pub fn set_legend(&mut self, legend: DisplayPositioned) {
        self.legend = legend;
    }

    #[must_use]
    pub fn get_legend(&self) -> &DisplayPositioned {
        &self.legend
    }

    // --- Header / Footer ---

    #[must_use]
    pub fn get_header(&self) -> &DisplayPositioned {
        &self.header
    }

    #[must_use]
    pub fn get_footer(&self) -> &DisplayPositioned {
        &self.footer
    }

    /// Updates the footer with a new display and alignment.
    ///
    /// Ported from: `TitledDiagram.updateFooter(...)`.
    pub fn update_footer(&mut self, display: Display, ha: HorizontalAlignment) {
        self.footer = self.footer.with_display(display).with_horizontal_alignment(ha);
    }

    /// Updates the header with a new display and alignment.
    ///
    /// Ported from: `TitledDiagram.updateHeader(...)`.
    pub fn update_header(&mut self, display: Display, ha: HorizontalAlignment) {
        self.header = self.header.with_display(display).with_horizontal_alignment(ha);
    }

    // --- Main Frame ---

    pub fn set_main_frame(&mut self, main_frame: DisplayPositioned) {
        self.main_frame = main_frame;
    }

    #[must_use]
    pub fn get_main_frame(&self) -> &DisplayPositioned {
        &self.main_frame
    }

    // --- Diagram Type ---

    #[must_use]
    pub const fn get_diagram_type(&self) -> DiagramType {
        self.diagram_type
    }

    // --- Skin Param ---

    #[must_use]
    pub fn get_skin_param(&self) -> &SkinParam {
        &self.skin_param
    }

    #[must_use]
    pub fn get_skin_param_mut(&mut self) -> &mut SkinParam {
        &mut self.skin_param
    }

    /// Sets a skin parameter.
    ///
    /// Ported from: `TitledDiagram.setParam(String, String)`.
    pub fn set_param(&mut self, key: &str, value: &str) {
        self.skin_param.set_param(&key.to_lowercase(), value);
        self.skin_param_used = true;
    }

    #[must_use]
    pub fn is_skin_param_used(&self) -> bool {
        self.skin_param_used
    }

    // --- Pragma ---

    #[must_use]
    pub fn get_pragma(&self) -> &Pragma {
        &self.pragma
    }

    #[must_use]
    pub fn get_pragma_mut(&mut self) -> &mut Pragma {
        &mut self.pragma
    }

    // --- Namespace Separator ---

    pub fn set_namespace_separator(&mut self, separator: impl Into<String>) {
        self.namespace_separator = Some(separator.into());
    }

    #[must_use]
    pub fn get_namespace_separator(&self) -> Option<&str> {
        self.namespace_separator.as_deref()
    }

    // --- Layout Flags ---

    pub fn set_use_smetana(&mut self, use_smetana: bool) {
        self.use_smetana = use_smetana;
    }

    #[must_use]
    pub fn is_use_smetana(&self) -> bool {
        FORCE_SMETANA.load(Ordering::Relaxed) || self.use_smetana
    }

    pub fn set_use_elk(&mut self, use_elk: bool) {
        self.use_elk = use_elk;
    }

    #[must_use]
    pub fn is_use_elk(&self) -> bool {
        FORCE_ELK.load(Ordering::Relaxed) || self.use_elk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_titled_diagram() {
        let td = TitledDiagram::new(DiagramType::Class);
        assert_eq!(td.get_diagram_type(), DiagramType::Class);
        assert!(td.get_title().is_null());
        assert!(td.get_caption().is_null());
        assert!(td.get_namespace_separator().is_none());
        assert!(!td.is_use_smetana());
        assert!(!td.is_use_elk());
    }

    #[test]
    fn test_set_title() {
        let mut td = TitledDiagram::new(DiagramType::State);
        let title = DisplayPositioned::single(
            Display::new("My Diagram"),
            HorizontalAlignment::Center,
            VerticalAlignment::Top,
        );
        td.set_title(title);
        assert!(!td.get_title().is_null());
        assert_eq!(td.get_title_display().size(), 1);
        assert_eq!(td.get_title_display().get(0), Some("My Diagram"));
    }

    #[test]
    fn test_set_title_ignores_white() {
        let mut td = TitledDiagram::new(DiagramType::State);
        let title = DisplayPositioned::single(
            Display::from_lines(["  ", ""]),
            HorizontalAlignment::Center,
            VerticalAlignment::Top,
        );
        td.set_title(title);
        assert!(td.get_title().is_null());
    }

    #[test]
    fn test_namespace_separator() {
        let mut td = TitledDiagram::new(DiagramType::Class);
        td.set_namespace_separator(".");
        assert_eq!(td.get_namespace_separator(), Some("."));
    }

    #[test]
    fn test_layout_flags() {
        let mut td = TitledDiagram::new(DiagramType::Class);
        td.set_use_smetana(true);
        assert!(td.is_use_smetana());
        td.set_use_elk(true);
        assert!(td.is_use_elk());
    }

    #[test]
    fn test_update_header_footer() {
        let mut td = TitledDiagram::new(DiagramType::Class);
        td.update_header(Display::new("Header"), HorizontalAlignment::Left);
        assert!(!td.get_header().is_null());
        assert_eq!(td.get_header().get_display().get(0), Some("Header"));

        td.update_footer(Display::new("Footer"), HorizontalAlignment::Right);
        assert!(!td.get_footer().is_null());
        assert_eq!(td.get_footer().get_display().get(0), Some("Footer"));
    }
}
