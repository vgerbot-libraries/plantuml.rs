//! ISkinParam — trait interface for skin parameters.
//!
//! Ported from: `net/sourceforge/plantuml/style/ISkinParam.java`
//!              `net/sourceforge/plantuml/style/ISkinSimple.java`

use std::collections::HashMap;

use plantuml_core::DiagramType;
use plantuml_core::UFont;
use plantuml_klimt::HColor;
use plantuml_klimt::HColorSet;

use crate::clockwise_top_right_bottom_left::ClockwiseTopRightBottomLeft;
use crate::length_adjust::LengthAdjust;
use crate::pragma::Pragma;
use crate::style::Style;
use crate::style_builder::StyleBuilder;
use crate::value::HorizontalAlignment;
pub use crate::placeholder_types::*;

// ---------------------------------------------------------------------------
// ISkinParam trait
// ---------------------------------------------------------------------------

/// Swimlane width sentinel: -1 means "same as other swimlanes".
pub const SWIMLANE_WIDTH_SAME: i32 = -1;

/// Trait interface for all skin parameters.
///
/// Ported from: `net/sourceforge/plantuml/style/ISkinParam.java`
///              `net/sourceforge/plantuml/style/ISkinSimple.java`
pub trait ISkinParam {
    /// Returns the value for a skin parameter key.
    fn get_value(&self, key: &str) -> Option<String>;

    /// Returns all parameter values.
    fn values(&self) -> HashMap<String, String>;

    /// Returns the padding.
    fn get_padding(&self) -> ClockwiseTopRightBottomLeft;

    /// Returns the monospaced font family name.
    fn get_monospaced_family(&self) -> String;

    /// Returns the tab size.
    fn get_tab_size(&self) -> i32;

    /// Returns the color set.
    fn get_i_html_color_set(&self) -> HColorSet;

    /// Returns the DPI.
    fn get_dpi(&self) -> i32;

    /// Copies all parameters from `other`.
    fn copy_all_from(&mut self, other: HashMap<String, String>);

    /// Returns the pragma.
    fn get_pragma(&self) -> &Pragma;

    // --- ISkinParam methods ---

    /// Returns the hyperlink color.
    fn get_hyperlink_color(&self) -> HColor;

    /// Returns the underline stroke for hyperlinks, if any.
    fn use_underline_for_hyperlink(&self) -> Option<UStroke>;

    /// Returns the background color.
    fn get_background_color(&self) -> HColor;

    /// Returns a color for a color parameter.
    fn get_html_color(
        &self,
        param: ColorParam,
        stereotype: Option<&Stereotype>,
        clickable: bool,
    ) -> Option<HColor>;

    /// Returns the font HTML color.
    fn get_font_html_color(
        &self,
        stereotype: Option<&Stereotype>,
        param: &[FontParam],
    ) -> Option<HColor>;

    /// Returns the line thickness.
    fn get_thickness(
        &self,
        param: LineParam,
        stereotype: Option<&Stereotype>,
    ) -> Option<UStroke>;

    /// Returns a font.
    fn get_font(
        &self,
        stereotype: Option<&Stereotype>,
        in_package_title: bool,
        param: &[FontParam],
    ) -> UFont;

    /// Returns the horizontal alignment for an alignment parameter.
    fn get_horizontal_alignment(
        &self,
        param: AlignmentParam,
        arrow_direction: ArrowDirection,
        is_reverse_define: bool,
        override_default: Option<HorizontalAlignment>,
    ) -> HorizontalAlignment;

    /// Returns the default text alignment.
    fn get_default_text_alignment(
        &self,
        default_value: HorizontalAlignment,
    ) -> HorizontalAlignment;

    /// Returns the stereotype alignment.
    fn get_stereotype_alignment(&self) -> HorizontalAlignment;

    /// Returns the circled character radius.
    fn get_circled_character_radius(&self) -> i32;

    /// Returns the circled character for a stereotype.
    fn get_circled_character(&self, stereotype: &Stereotype) -> char;

    /// Returns the class attribute icon size.
    fn class_attribute_icon_size(&self) -> i32;

    /// Returns the dot splines mode.
    fn get_dot_splines(&self) -> DotSplines;

    /// Returns whether shadowing is enabled.
    fn shadowing(&self, stereotype: Option<&Stereotype>) -> bool;

    /// Returns whether note shadowing is enabled.
    fn shadowing_for_note(&self, stereotype: Option<&Stereotype>) -> bool;

    /// Returns the package style.
    fn package_style(&self) -> PackageStyle;

    /// Returns the component style.
    fn component_style(&self) -> ComponentStyle;

    /// Returns whether stereotype position is top.
    fn stereotype_position_top(&self) -> bool;

    /// Returns whether swimlanes are used.
    fn use_swimlanes(&self, diagram_type: DiagramType) -> bool;

    /// Returns the node separation.
    fn get_nodesep(&self) -> f64;

    /// Returns the rank separation.
    fn get_ranksep(&self) -> f64;

    /// Returns the round corner value.
    fn get_round_corner(&self, param: CornerParam, stereotype: Option<&Stereotype>) -> f64;

    /// Returns the diagonal corner value.
    fn get_diagonal_corner(&self, param: CornerParam, stereotype: Option<&Stereotype>) -> f64;

    /// Returns the max message size line break strategy.
    fn max_message_size(&self) -> LineBreakStrategy;

    /// Returns the swimlane wrap title width line break strategy.
    fn swimlane_wrap_title_width(&self) -> LineBreakStrategy;

    /// Returns whether strict UML style is enabled.
    fn strict_uml_style(&self) -> bool;

    /// Returns whether sequence participant is underlined.
    fn force_sequence_participant_underlined(&self) -> bool;

    /// Returns the condition style.
    fn get_condition_style(&self) -> ConditionStyle;

    /// Returns the condition end style.
    fn get_condition_end_style(&self) -> ConditionEndStyle;

    /// Returns whether all classes have the same width.
    fn same_class_width(&self) -> bool;

    /// Returns the rank direction.
    fn get_rankdir(&self) -> Rankdir;

    /// Returns whether octagon is used for activity shapes.
    fn use_octagon_for_activity(&self, stereotype: Option<&Stereotype>) -> bool;

    /// Returns the group inheritance limit.
    fn group_inheritance(&self) -> i32;

    /// Returns the guillemet style.
    fn guillemet(&self) -> Guillemet;

    /// Returns whether handwritten mode is enabled.
    fn handwritten(&self) -> bool;

    /// Returns the SVG link target.
    fn get_svg_link_target(&self) -> String;

    /// Returns the preserve aspect ratio.
    fn get_preserve_aspect_ratio(&self) -> String;

    /// Returns the tab size (ISkinSimple).
    fn get_tab_size_simple(&self) -> i32 {
        self.get_tab_size()
    }

    /// Returns the max ASCII message length.
    fn max_ascii_message_length(&self) -> i32;

    /// Returns the color arrow separation space.
    fn color_arrow_separation_space(&self) -> i32;

    /// Returns the split parameter.
    fn get_split_param(&self) -> SplitParam;

    /// Returns the swimlane width.
    fn swimlane_width(&self) -> i32;

    /// Returns the diagram type.
    fn get_diagram_type(&self) -> DiagramType;

    /// Returns the hover path color.
    fn hover_path_color(&self) -> Option<HColor>;

    /// Returns the Tikz font distortion.
    fn get_tikz_font_distortion(&self) -> TikzFontDistortion;

    /// Returns the padding for a padding parameter.
    fn get_padding_to_be_removed(&self, param: PaddingParam) -> ClockwiseTopRightBottomLeft;

    /// Returns whether rank same is used.
    fn use_rank_same(&self) -> bool;

    /// Returns whether generic types are displayed with old fashion.
    fn display_generic_with_old_fashion(&self) -> bool;

    /// Returns whether response message is below arrow.
    fn response_message_below_arrow(&self) -> bool;

    /// Returns whether SVG dimension style is enabled.
    fn svg_dimension_style(&self) -> bool;

    /// Returns whether circle label overlapping is fixed.
    fn fix_circle_label_overlapping(&self) -> bool;

    /// Sets whether VizJs is used.
    fn set_use_vizjs(&mut self, use_vizjs: bool);

    /// Returns whether VizJs is used.
    fn is_use_vizjs(&self) -> bool;

    /// Returns the sequence diagram padder.
    fn sequence_diagram_padder(&self) -> Padder;

    /// Returns the current style builder.
    fn get_current_style_builder(&self) -> &StyleBuilder;

    /// Mutes the style with modified styles.
    fn mute_style(&mut self, modified_styles: &[Style]);

    /// Returns all sprite names.
    fn get_all_sprite_names(&self) -> Vec<String>;

    /// Returns the default skin name.
    fn get_default_skin(&self) -> String;

    /// Sets the default skin name.
    fn set_default_skin(&mut self, new_skin: String);

    /// Returns the actor style.
    fn actor_style(&self) -> ActorStyle;

    /// Sets the SVG size.
    fn set_svg_size(&mut self, origin: &str, size_to_use: &str);

    /// Returns the length adjust.
    fn get_length_adjust(&self) -> LengthAdjust;

    /// Returns the param same class width.
    fn get_param_same_class_width(&self) -> f64;

    /// Returns the arrows.
    fn arrows(&self) -> Arrows;
}
