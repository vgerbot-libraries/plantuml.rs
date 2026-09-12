//! PlantUML skin, style, and theme system.
//!
//! Ported from: `net/sourceforge/plantuml/skin/` and `net/sourceforge/plantuml/style/` packages.

pub mod arrow;
pub mod clockwise_top_right_bottom_left;
pub mod component;
pub mod is_skin_param;
pub mod placeholder_types;
pub mod level_constraint;
pub mod length_adjust;
pub mod merge_strategy;
pub mod p_name;
pub mod pragma;
pub mod pragma_key;
pub mod s_name;
pub mod skin_param;
mod skin_param_helpers;
pub mod specificity;
pub mod style;
pub mod style_builder;
pub mod style_query;
pub mod value;

pub use is_skin_param::{ISkinParam, SWIMLANE_WIDTH_SAME};
pub use placeholder_types::{
    ActorStyle, AlignmentParam, Arrows, ArrowDirection, ColorParam, Colors, ComponentStyle,
    ConditionEndStyle, ConditionStyle, CornerParam, DotSplines, FontParam, Guillemet,
    LineBreakStrategy, LineParam, PaddingParam, Padder, PackageStyle, Rankdir, SplitParam,
    Stereotype, TikzFontDistortion, UStroke,
};
pub use length_adjust::LengthAdjust;
pub use merge_strategy::MergeStrategy;
pub use p_name::PName;
pub use pragma::{Pragma, Warning};
pub use pragma_key::PragmaKey;
pub use s_name::SName;
pub use skin_param::{SkinParam, DEFAULT_PRESERVE_ASPECT_RATIO, DEFAULT_SKIN};
pub use specificity::Specificity;
pub use style::{Style, STAR, ID_TITLE, ID_CAPTION, ID_LEGEND};
pub use style_builder::{AutomaticCounter, StyleBuilder};
pub use arrow::{
    ArrowBody, ArrowConfiguration, ArrowDecoration, ArrowDressing, ArrowHead, ArrowPart,
};
pub use component::{Area, ComponentType, Context2D, SimpleContext2D};
// Re-export alignment types from klimt (canonical implementation).
pub use plantuml_klimt::{HorizontalAlignment, VerticalAlignment};
pub use style_query::{StyleAtom, StyleQuery};
pub use value::{DarkString, UFontFace, Value};
