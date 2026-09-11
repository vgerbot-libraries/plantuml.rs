//! SkinParam — the main skin parameter implementation.
//!
//! Ported from: `net/sourceforge/plantuml/skin/SkinParam.java`

use std::collections::HashMap;

use plantuml_core::DiagramType;
use plantuml_core::UFont;
use plantuml_core::u_font::FontStyle;
use plantuml_klimt::HColor;
use plantuml_klimt::HColorSet;
use plantuml_klimt::HColors;

use crate::clockwise_top_right_bottom_left::ClockwiseTopRightBottomLeft;
use crate::is_skin_param::{
    AlignmentParam, ActorStyle, Arrows, ArrowDirection, ColorParam, ComponentStyle, ConditionEndStyle,
    ConditionStyle, CornerParam, DotSplines, FontParam, Guillemet, ISkinParam, LineBreakStrategy,
    LineParam, PaddingParam, Padder, PackageStyle, Rankdir, SplitParam, Stereotype, TikzFontDistortion,
    UStroke, SWIMLANE_WIDTH_SAME,
};
use crate::length_adjust::LengthAdjust;
use crate::pragma::Pragma;
use crate::style::Style;
use crate::style_builder::StyleBuilder;
use crate::value::HorizontalAlignment;
use crate::skin_param_helpers::{is_digits, is_digits_or_dot, is_int_or_decimal, remove_quotes};

/// Default preserve aspect ratio.
pub const DEFAULT_PRESERVE_ASPECT_RATIO: &str = "none";

/// Default skin file name.
pub const DEFAULT_SKIN: &str = "plantuml.skin";

/// The main skin parameter implementation.
///
/// Ported from: `net/sourceforge/plantuml/skin/SkinParam.java`
#[derive(Debug, Clone)]
pub struct SkinParam {
    params: HashMap<String, String>,
    pragma: Pragma,
    rankdir: Rankdir,
    diagram_type: DiagramType,
    use_vizjs: bool,
    skin: String,
    style_builder: StyleBuilder,
    param_same_class_width: f64,
    svg_char_sizes: HashMap<String, String>,
}

impl Default for SkinParam {
    fn default() -> Self {
        Self::new(DiagramType::Unknown, Pragma::create_empty())
    }
}

impl SkinParam {
    /// Creates a new SkinParam for the given diagram type and pragma.
    #[must_use]
    pub fn new(diagram_type: DiagramType, pragma: Pragma) -> Self {
        Self {
            params: HashMap::new(),
            pragma,
            rankdir: Rankdir::TopToBottom,
            diagram_type,
            use_vizjs: false,
            skin: DEFAULT_SKIN.to_string(),
            style_builder: StyleBuilder::new(),
            param_same_class_width: 0.0,
            svg_char_sizes: HashMap::new(),
        }
    }
    /// Returns `true` if the skin param is in dark mode.
    #[must_use]
    pub fn is_dark(skin_param: &dyn ISkinParam) -> bool {
        skin_param
            .get_value("mode")
            .map(|v| v.eq_ignore_ascii_case("dark"))
            .unwrap_or(false)
    }
    /// Sets a parameter value.
    ///
    /// Ported from: `SkinParam.setParam(String, String)`.
    pub fn set_param(&mut self, key: &str, value: &str) {
        let cleaned = key.to_lowercase();
        self.params.insert(cleaned, value.trim().to_string());
    }
    /// Returns `true` if the key's value equals `expected` (case-insensitive).
    fn value_is(&self, key: &str, expected: &str) -> bool {
        self.get_value(key)
            .map(|v| v.eq_ignore_ascii_case(expected))
            .unwrap_or(false)
    }
    /// Returns `true` if the key's value is "true".
    fn is_true(&self, key: &str) -> bool {
        self.value_is(key, "true")
    }
    /// Returns the value as an integer, or `default_value` if not parseable.
    fn get_as_int(&self, key: &str, default_value: i32) -> i32 {
        self.get_value(key)
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(default_value)
    }
    /// Returns a `ClockwiseTopRightBottomLeft` from a numeric parameter.
    fn get_as_double(&self, name: &str) -> ClockwiseTopRightBottomLeft {
        match self.get_value(name) {
            Some(v) if is_int_or_decimal(&v) => {
                ClockwiseTopRightBottomLeft::same(v.parse::<f64>().unwrap_or(0.0))
            }
            _ => ClockwiseTopRightBottomLeft::same(0.0),
        }
    }
    /// Returns the first non-null value with a suffix for the given font params.
    fn get_first_value_non_null_with_suffix(
        &self,
        suffix: &str,
        param: &[FontParam],
    ) -> Option<String> {
        for p in param {
            let key = format!("{}{}", p.name().to_lowercase(), suffix);
            if let Some(v) = self.get_value(&key) {
                return Some(v);
            }
        }
        None
    }
    /// Returns the font family for the given stereotype and font params.
    fn get_font_family(&self, stereotype: Option<&Stereotype>, param: &[FontParam]) -> String {
        if let Some(stereo) = stereotype {
            let suffix = format!("fontname{}", stereo.get_label("<<"));
            if let Some(v) = self.get_first_value_non_null_with_suffix(&suffix, param) {
                return remove_quotes(&v);
            }
        }
        if let Some(v) = self.get_first_value_non_null_with_suffix("fontname", param) {
            return remove_quotes(&v);
        }
        if param.first().is_none_or(|p| *p != FontParam::CircledCharacter) {
            if let Some(v) = self.get_value("defaultfontname") {
                return remove_quotes(&v);
            }
        }
        param
            .first()
            .map_or("sans-serif", |p| p.default_family())
            .to_string()
    }
    /// Returns the font size for the given stereotype and font params.
    fn get_font_size(&self, stereotype: Option<&Stereotype>, param: &[FontParam]) -> i32 {
        if let Some(stereo) = stereotype {
            let suffix = format!("fontsize{}", stereo.get_label("<<"));
            if let Some(v) = self.get_first_value_non_null_with_suffix(&suffix, param) {
                if is_digits(&v) {
                    return v.parse().unwrap_or(14);
                }
            }
        }
        let mut value = self.get_first_value_non_null_with_suffix("fontsize", param);
        if !value.as_deref().is_some_and(is_digits) {
            value = self.get_value("defaultfontsize");
        }
        if !value.as_deref().is_some_and(is_digits) {
            return param.first().map_or(14, |p| p.default_size());
        }
        value.and_then(|v| v.parse().ok()).unwrap_or(14)
    }
    /// Returns the font HTML color value for the given stereotype and font params.
    fn get_font_html_color_value(
        &self,
        stereotype: Option<&Stereotype>,
        param: &[FontParam],
    ) -> Option<String> {
        let mut value = None;
        if let Some(stereo) = stereotype {
            let suffix = format!("fontcolor{}", stereo.get_label("<<"));
            value = self.get_first_value_non_null_with_suffix(&suffix, param);
        }
        if value.is_none() {
            value = self.get_first_value_non_null_with_suffix("fontcolor", param);
        }
        if value.is_none() {
            value = self.get_value("defaultfontcolor");
        }
        if value.is_none() {
            value = param.first().and_then(|p| p.default_color().map(String::from));
        }
        value
    }
    /// Sets the param same class width.
    pub fn set_param_same_class_width(&mut self, width: f64) {
        self.param_same_class_width = width;
    }
    /// Sets the rank direction.
    pub fn set_rankdir(&mut self, rankdir: Rankdir) {
        self.rankdir = rankdir;
    }
}

impl ISkinParam for SkinParam {
    fn get_value(&self, key: &str) -> Option<String> {
        self.params.get(&key.to_lowercase()).cloned()
    }
    fn values(&self) -> HashMap<String, String> {
        self.params.clone()
    }
    fn get_padding(&self) -> ClockwiseTopRightBottomLeft {
        self.get_as_double("padding")
    }
    fn get_monospaced_family(&self) -> String {
        self.get_value("defaultmonospacedfontname")
            .unwrap_or_else(|| "monospace".to_string())
    }
    fn get_tab_size(&self) -> i32 {
        self.get_as_int("tabsize", 8)
    }
    fn get_i_html_color_set(&self) -> HColorSet {
        HColorSet
    }
    fn get_dpi(&self) -> i32 {
        let dpi = self.get_as_int("dpi", 96);
        if dpi <= 0 {
            96
        } else {
            dpi
        }
    }
    fn copy_all_from(&mut self, other: HashMap<String, String>) {
        for (k, v) in other {
            self.params.insert(k.to_lowercase(), v);
        }
    }
    fn get_pragma(&self) -> &Pragma {
        &self.pragma
    }
    fn get_hyperlink_color(&self) -> HColor {
        self.get_html_color(ColorParam::Hyperlink, None, false)
            .unwrap_or_else(HColors::blue)
    }
    fn use_underline_for_hyperlink(&self) -> Option<UStroke> {
        if !self.value_is("hyperlinkunderline", "false") {
            Some(UStroke::simple())
        } else {
            None
        }
    }
    fn get_background_color(&self) -> HColor {
        if let Some(color) = self.get_html_color(ColorParam::Background, None, false) {
            return color;
        }
        HColors::white()
    }
    fn get_html_color(
        &self,
        param: ColorParam,
        stereotype: Option<&Stereotype>,
        _clickable: bool,
    ) -> Option<HColor> {
        if let Some(stereo) = stereotype {
            for s in stereo.get_multiple_labels() {
                let key = format!("{}color<<{}>>", param.name(), s);
                if let Some(v) = self.get_value(&key) {
                    return Some(HColorSet::get_color_or_white(&v));
                }
            }
        }
        let key = format!("{}color", param.name());
        let value = self.get_value(&key)?;
        if (param == ColorParam::Background || param == ColorParam::ArrowHead)
            && (value.eq_ignore_ascii_case("transparent") || value.eq_ignore_ascii_case("none"))
        {
            return Some(HColors::transparent());
        }
        Some(HColorSet::get_color_or_white(&value))
    }
    fn get_font_html_color(
        &self,
        stereotype: Option<&Stereotype>,
        param: &[FontParam],
    ) -> Option<HColor> {
        let value = self.get_font_html_color_value(stereotype, param)?;
        Some(HColorSet::get_color_or_white(&value))
    }
    fn get_thickness(
        &self,
        param: LineParam,
        stereotype: Option<&Stereotype>,
    ) -> Option<UStroke> {
        if let Some(stereo) = stereotype {
            let suffix = format!("thickness{}", stereo.get_label("<<"));
            let key = format!("{}{}", param.name(), suffix);
            if let Some(v) = self.get_value(&key) {
                if is_digits_or_dot(&v) {
                    return Some(UStroke::with_thickness(v.parse().unwrap_or(1.0)));
                }
            }
        }
        let key = format!("{}thickness", param.name());
        if let Some(v) = self.get_value(&key) {
            if is_digits_or_dot(&v) {
                return Some(UStroke::with_thickness(v.parse().unwrap_or(1.0)));
            }
        }
        None
    }
    fn get_font(
        &self,
        stereotype: Option<&Stereotype>,
        _in_package_title: bool,
        param: &[FontParam],
    ) -> UFont {
        let family = self.get_font_family(stereotype, param);
        let size = self.get_font_size(stereotype, param);
        let style = self.get_font_style(stereotype, param);
        UFont::new(family, style, size)
    }
    fn get_horizontal_alignment(
        &self,
        param: AlignmentParam,
        arrow_direction: ArrowDirection,
        is_reverse_define: bool,
        override_default: Option<HorizontalAlignment>,
    ) -> HorizontalAlignment {
        let value = match param {
            AlignmentParam::SequenceMessageAlignment => {
                self.get_value(AlignmentParam::SequenceMessageAlignment.name())
            }
            AlignmentParam::SequenceMessageTextAlignment => {
                self.get_value(AlignmentParam::SequenceMessageAlignment.name())
            }
            _ => self.get_value(param.name()),
        };

        if let Some(ref v) = value {
            if v.eq_ignore_ascii_case("first") {
                if arrow_direction == ArrowDirection::RightToLeftReverse {
                    return if is_reverse_define {
                        HorizontalAlignment::Left
                    } else {
                        HorizontalAlignment::Right
                    };
                }
                return if is_reverse_define {
                    HorizontalAlignment::Right
                } else {
                    HorizontalAlignment::Left
                };
            }
            if v.eq_ignore_ascii_case("direction") {
                return match arrow_direction {
                    ArrowDirection::LeftToRightNormal => HorizontalAlignment::Left,
                    ArrowDirection::RightToLeftReverse => HorizontalAlignment::Right,
                    ArrowDirection::BothDirection => HorizontalAlignment::Center,
                    ArrowDirection::Self_ => HorizontalAlignment::Center,
                };
            }
            if v.eq_ignore_ascii_case("reversedirection") {
                return match arrow_direction {
                    ArrowDirection::LeftToRightNormal => HorizontalAlignment::Right,
                    ArrowDirection::RightToLeftReverse => HorizontalAlignment::Left,
                    ArrowDirection::BothDirection => HorizontalAlignment::Center,
                    ArrowDirection::Self_ => HorizontalAlignment::Center,
                };
            }
        }

        if let Some(result) = value.as_deref().and_then(HorizontalAlignment::from_string) {
            return result;
        }

        if param == AlignmentParam::NoteTextAlignment {
            return self.get_default_text_alignment(override_default.unwrap_or(HorizontalAlignment::Left));
        }
        if param == AlignmentParam::StateMessageAlignment {
            return self.get_default_text_alignment(HorizontalAlignment::Center);
        }
        param.get_default_value()
    }
    fn get_default_text_alignment(&self, default_value: HorizontalAlignment) -> HorizontalAlignment {
        match self.get_value("defaulttextalignment") {
            Some(v) => HorizontalAlignment::from_string(&v).unwrap_or(default_value),
            None => default_value,
        }
    }
    fn get_stereotype_alignment(&self) -> HorizontalAlignment {
        match self.get_value("stereotypealignment") {
            Some(v) => HorizontalAlignment::from_string(&v).unwrap_or(HorizontalAlignment::Center),
            None => HorizontalAlignment::Center,
        }
    }
    fn get_circled_character_radius(&self) -> i32 {
        let value = self.get_as_int("circledcharacterradius", -1);
        if value == -1 {
            self.get_font_size(None, &[FontParam::CircledCharacter]) / 3 + 6
        } else {
            value
        }
    }
    fn get_circled_character(&self, stereotype: &Stereotype) -> char {
        let key = format!("spotchar{}", stereotype.get_label("<<"));
        if let Some(v) = self.get_value(&key) {
            if let Some(c) = v.chars().next() {
                return c;
            }
        }
        '\0'
    }
    fn class_attribute_icon_size(&self) -> i32 {
        self.get_as_int("classattributeiconsize", 10)
    }
    fn get_dot_splines(&self) -> DotSplines {
        match self.get_value("linetype") {
            Some(v) if v.eq_ignore_ascii_case("polyline") => DotSplines::Polyline,
            Some(v) if v.eq_ignore_ascii_case("ortho") => DotSplines::Ortho,
            _ => DotSplines::Splines,
        }
    }
    fn shadowing(&self, stereotype: Option<&Stereotype>) -> bool {
        if let Some(stereo) = stereotype {
            let key = format!("shadowing{}", stereo.get_label("<<"));
            if let Some(v) = self.get_value(&key) {
                return v.eq_ignore_ascii_case("true");
            }
        }
        let value = self.get_value("shadowing");
        if value.as_deref().is_some_and(|v| v.eq_ignore_ascii_case("false")) {
            return false;
        }
        if value.as_deref().is_some_and(|v| v.eq_ignore_ascii_case("true")) {
            return true;
        }
        if self.strict_uml_style() {
            return false;
        }
        true
    }
    fn shadowing_for_note(&self, stereotype: Option<&Stereotype>) -> bool {
        if let Some(stereo) = stereotype {
            let key = format!("noteshadowing{}", stereo.get_label("<<"));
            if let Some(v) = self.get_value(&key) {
                return v.eq_ignore_ascii_case("true");
            }
        }
        if let Some(v) = self.get_value("noteshadowing") {
            return v.eq_ignore_ascii_case("true");
        }
        self.shadowing(stereotype)
    }
    fn package_style(&self) -> PackageStyle {
        self.get_value("packagestyle")
            .as_deref()
            .and_then(PackageStyle::from_string)
            .unwrap_or(PackageStyle::Folder)
    }
    fn component_style(&self) -> ComponentStyle {
        if self.strict_uml_style() {
            return ComponentStyle::Uml2;
        }
        match self.get_value("componentstyle") {
            Some(v) if v.eq_ignore_ascii_case("uml1") => ComponentStyle::Uml1,
            Some(v) if v.eq_ignore_ascii_case("uml2") => ComponentStyle::Uml2,
            Some(v) if v.eq_ignore_ascii_case("rectangle") => ComponentStyle::Rectangle,
            _ => ComponentStyle::Uml2,
        }
    }
    fn stereotype_position_top(&self) -> bool {
        !self.value_is("stereotypeposition", "bottom")
    }
    fn use_swimlanes(&self, diagram_type: DiagramType) -> bool {
        if diagram_type != DiagramType::Activity {
            return false;
        }
        self.is_true("swimlane") || self.is_true("swimlanes")
    }
    fn get_nodesep(&self) -> f64 {
        f64::from(self.get_as_int("nodesep", 0))
    }
    fn get_ranksep(&self) -> f64 {
        f64::from(self.get_as_int("ranksep", 0))
    }
    fn get_round_corner(&self, param: CornerParam, stereotype: Option<&Stereotype>) -> f64 {
        let key = param.get_round_key();
        if let Some(result) = self.get_corner_internal(key, stereotype) {
            return result;
        }
        if param == CornerParam::Default {
            return 0.0;
        }
        self.get_round_corner(CornerParam::Default, stereotype)
    }
    fn get_diagonal_corner(&self, param: CornerParam, stereotype: Option<&Stereotype>) -> f64 {
        let key = param.get_diagonal_key();
        if let Some(result) = self.get_corner_internal(key, stereotype) {
            return result;
        }
        if param == CornerParam::Default {
            return 0.0;
        }
        self.get_diagonal_corner(CornerParam::Default, stereotype)
    }
    fn max_message_size(&self) -> LineBreakStrategy {
        let value = self
            .get_value("wrapmessagewidth")
            .or_else(|| self.get_value("maxmessagesize"));
        LineBreakStrategy::new(value)
    }
    fn swimlane_wrap_title_width(&self) -> LineBreakStrategy {
        LineBreakStrategy::new(self.get_value("swimlanewraptitlewidth"))
    }
    fn strict_uml_style(&self) -> bool {
        self.value_is("style", "strictuml")
    }
    fn force_sequence_participant_underlined(&self) -> bool {
        self.value_is("sequenceparticipant", "underline")
    }
    fn get_condition_style(&self) -> ConditionStyle {
        self.get_value("conditionstyle")
            .as_deref()
            .and_then(ConditionStyle::from_string)
            .unwrap_or(ConditionStyle::InsideHexagon)
    }
    fn get_condition_end_style(&self) -> ConditionEndStyle {
        self.get_value("conditionendstyle")
            .as_deref()
            .and_then(ConditionEndStyle::from_string)
            .unwrap_or(ConditionEndStyle::Diamond)
    }
    fn same_class_width(&self) -> bool {
        self.is_true("sameclasswidth")
    }
    fn get_rankdir(&self) -> Rankdir {
        self.rankdir
    }
    fn use_octagon_for_activity(&self, stereotype: Option<&Stereotype>) -> bool {
        let mut value = self.get_value("activityshape");
        if let Some(stereo) = stereotype {
            let key = format!("activityshape{}", stereo.get_label("<<"));
            if let Some(v) = self.get_value(&key) {
                value = Some(v);
            }
        }
        match value {
            Some(v) if v.eq_ignore_ascii_case("roundedbox") => false,
            Some(v) if v.eq_ignore_ascii_case("octagon") => true,
            _ => false,
        }
    }
    fn group_inheritance(&self) -> i32 {
        let value = self.get_as_int("groupinheritance", i32::MAX);
        if value <= 1 {
            i32::MAX
        } else {
            value
        }
    }
    fn guillemet(&self) -> Guillemet {
        Guillemet::from_description(self.get_value("guillemet").as_deref())
    }
    fn handwritten(&self) -> bool {
        self.is_true("handwritten")
    }
    fn get_svg_link_target(&self) -> String {
        self.get_value("svglinktarget").unwrap_or_else(|| "_top".to_string())
    }
    fn get_preserve_aspect_ratio(&self) -> String {
        self.get_value("preserveaspectratio")
            .unwrap_or_else(|| DEFAULT_PRESERVE_ASPECT_RATIO.to_string())
    }
    fn max_ascii_message_length(&self) -> i32 {
        self.get_as_int("maxasciimessagelength", -1)
    }
    fn color_arrow_separation_space(&self) -> i32 {
        self.get_as_int("colorarrowseparationspace", 0)
    }
    fn get_split_param(&self) -> SplitParam {
        let border = self.get_value("pagebordercolor");
        let external = self.get_value("pageexternalcolor");
        let border_color = border.map(|v| HColorSet::get_color_or_white(&v));
        let external_color = external.map(|v| HColorSet::get_color_or_white(&v));
        let margin = self.get_as_int("pagemargin", 0);
        SplitParam {
            border_color,
            external_color,
            margin,
        }
    }
    fn swimlane_width(&self) -> i32 {
        let value = self.get_value("swimlanewidth");
        if value.as_deref().is_some_and(|v| v.eq_ignore_ascii_case("same")) {
            return SWIMLANE_WIDTH_SAME;
        }
        if let Some(ref v) = value {
            if is_digits(v) {
                return v.parse().unwrap_or(0);
            }
        }
        0
    }
    fn get_diagram_type(&self) -> DiagramType {
        self.diagram_type
    }
    fn hover_path_color(&self) -> Option<HColor> {
        self.get_value("pathhovercolor").map(|v| HColorSet::get_color_or_white(&v))
    }
    fn get_tikz_font_distortion(&self) -> TikzFontDistortion {
        TikzFontDistortion::from_value(self.get_value("tikzfont").as_deref())
    }
    fn get_padding_to_be_removed(&self, param: PaddingParam) -> ClockwiseTopRightBottomLeft {
        self.get_as_double(param.get_skin_name())
    }
    fn use_rank_same(&self) -> bool {
        false
    }
    fn display_generic_with_old_fashion(&self) -> bool {
        self.value_is("genericdisplay", "old")
    }
    fn response_message_below_arrow(&self) -> bool {
        self.is_true("responsemessagebelowarrow")
    }
    fn svg_dimension_style(&self) -> bool {
        !self.value_is("svgdimensionstyle", "false")
    }
    fn fix_circle_label_overlapping(&self) -> bool {
        self.is_true("fixcirclelabeloverlapping")
    }
    fn set_use_vizjs(&mut self, use_vizjs: bool) {
        self.use_vizjs = use_vizjs;
    }
    fn is_use_vizjs(&self) -> bool {
        self.use_vizjs
    }
    fn sequence_diagram_padder(&self) -> Padder {
        let padding = self.get_as_double("sequencemessagepadding");
        let margin = self.get_as_double("sequencemessagemargin");
        let border_color = self
            .get_value("sequencemessagebordercolor")
            .map(|v| HColorSet::get_color_or_white(&v));
        let background_color = self
            .get_value("sequencemessagebackgroundcolor")
            .map(|v| HColorSet::get_color_or_white(&v));
        if padding.is_zero() && margin.is_zero() && border_color.is_none() && background_color.is_none() {
            return Padder::NONE;
        }
        let round_corner = self.get_round_corner(CornerParam::Default, None);
        Padder {
            padding,
            margin,
            border_color,
            background_color,
            round_corner,
        }
    }
    fn get_current_style_builder(&self) -> &StyleBuilder {
        &self.style_builder
    }
    fn mute_style(&mut self, modified_styles: &[Style]) {
        self.style_builder = self.style_builder.mute_style(modified_styles);
    }
    fn get_all_sprite_names(&self) -> Vec<String> {
        Vec::new()
    }
    fn get_default_skin(&self) -> String {
        self.skin.clone()
    }
    fn set_default_skin(&mut self, new_skin: String) {
        self.skin = new_skin;
    }
    fn actor_style(&self) -> ActorStyle {
        match self.get_value("actorstyle") {
            Some(v) if v.eq_ignore_ascii_case("awesome") => ActorStyle::Awesome,
            Some(v) if v.eq_ignore_ascii_case("hollow") => ActorStyle::Hollow,
            _ => ActorStyle::Stickman,
        }
    }
    fn set_svg_size(&mut self, origin: &str, size_to_use: &str) {
        self.svg_char_sizes.insert(origin.to_string(), size_to_use.to_string());
    }
    fn get_length_adjust(&self) -> LengthAdjust {
        match self.get_value("lengthadjust") {
            Some(v) if v.eq_ignore_ascii_case("spacingandglyphs") => LengthAdjust::SpacingAndGlyphs,
            Some(v) if v.eq_ignore_ascii_case("spacing") => LengthAdjust::Spacing,
            Some(v) if v.eq_ignore_ascii_case("none") => LengthAdjust::None,
            _ => LengthAdjust::default_value(),
        }
    }
    fn get_param_same_class_width(&self) -> f64 {
        self.param_same_class_width
    }
    fn arrows(&self) -> Arrows {
        Arrows
    }
}

impl SkinParam {
    /// Returns the font style (bold/italic/plain) for the given stereotype and
    /// font params.
    fn get_font_style(&self, stereotype: Option<&Stereotype>, param: &[FontParam]) -> FontStyle {
        let mut value = None;
        if let Some(stereo) = stereotype {
            let suffix = format!("fontstyle{}", stereo.get_label("<<"));
            value = self.get_first_value_non_null_with_suffix(&suffix, param);
        }
        if value.is_none() {
            value = self.get_first_value_non_null_with_suffix("fontstyle", param);
        }
        if value.is_none() {
            value = self.get_value("defaultfontstyle");
        }
        let value = match value {
            Some(v) => v,
            None => return FontStyle::plain(),
        };
        let lower = value.to_lowercase();
        let bold = lower.contains("bold");
        let italic = lower.contains("italic");
        if bold && italic {
            FontStyle::bold_italic()
        } else if bold {
            FontStyle::bold()
        } else if italic {
            FontStyle::italic()
        } else {
            FontStyle::plain()
        }
    }
    /// Returns a corner value from the internal parameter store.
    fn get_corner_internal(
        &self,
        key: &str,
        stereotype: Option<&Stereotype>,
    ) -> Option<f64> {
        let full_key = match stereotype {
            Some(stereo) => format!("{}{}", key, stereo.get_label("<<")),
            None => key.to_string(),
        };
        let value = self.get_value(&full_key)?;
        if is_digits(&value) {
            Some(value.parse().unwrap_or(0.0))
        } else {
            None
        }
    }
}
