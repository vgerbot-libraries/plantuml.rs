//! Foundational types shared by all plantuml crates.
//!
//! Ported from the `net.sourceforge.plantuml` root and `core` packages.

pub mod diagram;
pub mod diagram_description;
pub mod diagram_type;
pub mod error;
pub mod file_format;
pub mod p_system_error;
pub mod file_format_option;
pub mod geom;
pub mod string_bounder;
pub mod text_block;
pub mod u_font;

pub use diagram::Diagram;
pub use diagram_description::DiagramDescription;
pub use diagram_type::DiagramType;
pub use error::PlantumlError;
pub use p_system_error::{ErrorUmlType, PSystemError};
pub use file_format::FileFormat;
pub use file_format_option::FileFormatOption;
pub use geom::{XDimension2D, XLine2D, XPoint2D, XRectangle2D};
pub use string_bounder::StringBounder;
pub use text_block::TextBlock;
pub use u_font::UFont;
