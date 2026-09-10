//! Integration tests for SkinParam defaults and parameter handling.
//!
//! Ported from the test section of `net/sourceforge/plantuml/skin/SkinParam.java`

use std::collections::HashMap;

use plantuml_core::DiagramType;
use plantuml_core::u_font::UFontContext;
use plantuml_klimt::HColors;
use plantuml_skin::{
    ActorStyle, ComponentStyle, ConditionEndStyle, ConditionStyle, CornerParam, DotSplines,
    FontParam, ISkinParam, LengthAdjust, PackageStyle, Pragma, Rankdir, SkinParam, SWIMLANE_WIDTH_SAME,
};

#[test]
fn default_background_color() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_background_color(), HColors::white());
}

#[test]
fn default_shadowing() {
    let sp = SkinParam::default();
    assert!(sp.shadowing(None));
}

#[test]
fn strict_uml_disables_shadowing() {
    let mut sp = SkinParam::default();
    sp.set_param("style", "strictuml");
    assert!(!sp.shadowing(None));
}

#[test]
fn set_and_get_param() {
    let mut sp = SkinParam::default();
    sp.set_param("shadowing", "false");
    assert!(!sp.shadowing(None));
}

#[test]
fn default_dpi() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_dpi(), 96);
}

#[test]
fn custom_dpi() {
    let mut sp = SkinParam::default();
    sp.set_param("dpi", "300");
    assert_eq!(sp.get_dpi(), 300);
}

#[test]
fn default_tab_size() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_tab_size(), 8);
}

#[test]
fn default_package_style() {
    let sp = SkinParam::default();
    assert_eq!(sp.package_style(), PackageStyle::Folder);
}

#[test]
fn default_component_style() {
    let sp = SkinParam::default();
    assert_eq!(sp.component_style(), ComponentStyle::Uml2);
}

#[test]
fn default_actor_style() {
    let sp = SkinParam::default();
    assert_eq!(sp.actor_style(), ActorStyle::Stickman);
}

#[test]
fn default_rankdir() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_rankdir(), Rankdir::TopToBottom);
}

#[test]
fn set_rankdir() {
    let mut sp = SkinParam::default();
    sp.set_rankdir(Rankdir::LeftToRight);
    assert_eq!(sp.get_rankdir(), Rankdir::LeftToRight);
}

#[test]
fn default_get_value() {
    let sp = SkinParam::default();
    assert!(sp.get_value("nonexistent").is_none());
}

#[test]
fn set_param_stores_value() {
    let mut sp = SkinParam::default();
    sp.set_param("FooBar", "test");
    assert_eq!(sp.get_value("foobar"), Some("test".to_string()));
}

#[test]
fn default_preserve_aspect_ratio() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_preserve_aspect_ratio(), "none");
}

#[test]
fn default_svg_link_target() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_svg_link_target(), "_top");
}

#[test]
fn default_length_adjust() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_length_adjust(), LengthAdjust::Spacing);
}

#[test]
fn default_class_attribute_icon_size() {
    let sp = SkinParam::default();
    assert_eq!(sp.class_attribute_icon_size(), 10);
}

#[test]
fn default_group_inheritance() {
    let sp = SkinParam::default();
    assert_eq!(sp.group_inheritance(), i32::MAX);
}

#[test]
fn use_swimlanes_only_for_activity() {
    let sp = SkinParam::default();
    assert!(!sp.use_swimlanes(DiagramType::Sequence));
    assert!(!sp.use_swimlanes(DiagramType::Activity));
}

#[test]
fn swimlanes_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("swimlane", "true");
    assert!(sp.use_swimlanes(DiagramType::Activity));
}

#[test]
fn default_use_rank_same() {
    let sp = SkinParam::default();
    assert!(!sp.use_rank_same());
}

#[test]
fn default_same_class_width() {
    let sp = SkinParam::default();
    assert!(!sp.same_class_width());
}

#[test]
fn same_class_width_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("sameclasswidth", "true");
    assert!(sp.same_class_width());
}

#[test]
fn default_handwritten() {
    let sp = SkinParam::default();
    assert!(!sp.handwritten());
}

#[test]
fn default_strict_uml_style() {
    let sp = SkinParam::default();
    assert!(!sp.strict_uml_style());
}

#[test]
fn default_svg_dimension_style() {
    let sp = SkinParam::default();
    assert!(sp.svg_dimension_style());
}

#[test]
fn svg_dimension_style_disabled() {
    let mut sp = SkinParam::default();
    sp.set_param("svgdimensionstyle", "false");
    assert!(!sp.svg_dimension_style());
}

#[test]
fn default_dot_splines() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_dot_splines(), DotSplines::Splines);
}

#[test]
fn dot_splines_ortho() {
    let mut sp = SkinParam::default();
    sp.set_param("linetype", "ortho");
    assert_eq!(sp.get_dot_splines(), DotSplines::Ortho);
}

#[test]
fn default_stereotype_alignment() {
    let sp = SkinParam::default();
    assert_eq!(
        sp.get_stereotype_alignment(),
        plantuml_skin::HorizontalAlignment::Center
    );
}

#[test]
fn default_stereotype_position_top() {
    let sp = SkinParam::default();
    assert!(sp.stereotype_position_top());
}

#[test]
fn stereotype_position_bottom() {
    let mut sp = SkinParam::default();
    sp.set_param("stereotypePosition", "bottom");
    assert!(!sp.stereotype_position_top());
}

#[test]
fn default_condition_style() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_condition_style(), ConditionStyle::InsideHexagon);
}

#[test]
fn default_condition_end_style() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_condition_end_style(), ConditionEndStyle::Diamond);
}

#[test]
fn default_get_font() {
    let sp = SkinParam::default();
    let font = sp.get_font(None, false, &[FontParam::Default]);
    assert_eq!(font.family("", UFontContext::Svg), "sans-serif");
    assert_eq!(font.size(), 14);
}

#[test]
fn custom_font_name() {
    let mut sp = SkinParam::default();
    sp.set_param("defaultfontname", "\"Times New Roman\"");
    let font = sp.get_font(None, false, &[FontParam::Default]);
    assert_eq!(font.family("", UFontContext::Svg), "Times New Roman");
}

#[test]
fn custom_font_size() {
    let mut sp = SkinParam::default();
    sp.set_param("defaultfontsize", "20");
    let font = sp.get_font(None, false, &[FontParam::Default]);
    assert_eq!(font.size(), 20);
}

#[test]
fn custom_font_style_bold() {
    let mut sp = SkinParam::default();
    sp.set_param("defaultfontstyle", "bold");
    let font = sp.get_font(None, false, &[FontParam::Default]);
    assert!(font.style().bold);
    assert!(!font.style().italic);
}

#[test]
fn custom_font_style_bold_italic() {
    let mut sp = SkinParam::default();
    sp.set_param("defaultfontstyle", "bold italic");
    let font = sp.get_font(None, false, &[FontParam::Default]);
    assert!(font.style().bold);
    assert!(font.style().italic);
}

#[test]
fn default_circled_character_radius() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_circled_character_radius(), 10);
}

#[test]
fn custom_circled_character_radius() {
    let mut sp = SkinParam::default();
    sp.set_param("circledCharacterRadius", "25");
    assert_eq!(sp.get_circled_character_radius(), 25);
}

#[test]
fn default_round_corner() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_round_corner(CornerParam::Default, None), 0.0);
}

#[test]
fn custom_round_corner() {
    let mut sp = SkinParam::default();
    sp.set_param("roundcorner", "15");
    assert_eq!(sp.get_round_corner(CornerParam::Default, None), 15.0);
}

#[test]
fn default_nodesep() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_nodesep(), 0.0);
}

#[test]
fn custom_nodesep() {
    let mut sp = SkinParam::default();
    sp.set_param("nodesep", "50");
    assert_eq!(sp.get_nodesep(), 50.0);
}

#[test]
fn default_ranksep() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_ranksep(), 0.0);
}

#[test]
fn custom_ranksep() {
    let mut sp = SkinParam::default();
    sp.set_param("ranksep", "30");
    assert_eq!(sp.get_ranksep(), 30.0);
}

#[test]
fn default_swimlane_width() {
    let sp = SkinParam::default();
    assert_eq!(sp.swimlane_width(), 0);
}

#[test]
fn swimlane_width_same() {
    let mut sp = SkinParam::default();
    sp.set_param("swimlanewidth", "same");
    assert_eq!(sp.swimlane_width(), SWIMLANE_WIDTH_SAME);
}

#[test]
fn swimlane_width_custom() {
    let mut sp = SkinParam::default();
    sp.set_param("swimlanewidth", "200");
    assert_eq!(sp.swimlane_width(), 200);
}

#[test]
fn default_get_default_skin() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_default_skin(), "plantuml.skin");
}

#[test]
fn set_default_skin() {
    let mut sp = SkinParam::default();
    sp.set_default_skin("custom.skin".to_string());
    assert_eq!(sp.get_default_skin(), "custom.skin");
}

#[test]
fn default_use_vizjs() {
    let sp = SkinParam::default();
    assert!(!sp.is_use_vizjs());
}

#[test]
fn set_use_vizjs() {
    let mut sp = SkinParam::default();
    sp.set_use_vizjs(true);
    assert!(sp.is_use_vizjs());
}

#[test]
fn default_get_padding() {
    let sp = SkinParam::default();
    assert!(sp.get_padding().is_zero());
}

#[test]
fn custom_padding() {
    let mut sp = SkinParam::default();
    sp.set_param("padding", "10");
    let p = sp.get_padding();
    assert_eq!(p.get_top(), 10.0);
    assert_eq!(p.get_right(), 10.0);
}

#[test]
fn default_monospaced_family() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_monospaced_family(), "monospace");
}

#[test]
fn custom_monospaced_family() {
    let mut sp = SkinParam::default();
    sp.set_param("defaultMonospacedFontName", "Courier New");
    assert_eq!(sp.get_monospaced_family(), "Courier New");
}

#[test]
fn default_max_ascii_message_length() {
    let sp = SkinParam::default();
    assert_eq!(sp.max_ascii_message_length(), -1);
}

#[test]
fn default_color_arrow_separation_space() {
    let sp = SkinParam::default();
    assert_eq!(sp.color_arrow_separation_space(), 0);
}

#[test]
fn default_use_octagon_for_activity() {
    let sp = SkinParam::default();
    assert!(!sp.use_octagon_for_activity(None));
}

#[test]
fn octagon_for_activity() {
    let mut sp = SkinParam::default();
    sp.set_param("activityshape", "octagon");
    assert!(sp.use_octagon_for_activity(None));
}

#[test]
fn default_hyperlink_color() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_hyperlink_color(), HColors::blue());
}

#[test]
fn default_use_underline_for_hyperlink() {
    let sp = SkinParam::default();
    assert!(sp.use_underline_for_hyperlink().is_some());
}

#[test]
fn disable_underline_for_hyperlink() {
    let mut sp = SkinParam::default();
    sp.set_param("hyperlinkunderline", "false");
    assert!(sp.use_underline_for_hyperlink().is_none());
}

#[test]
fn default_force_sequence_participant_underlined() {
    let sp = SkinParam::default();
    assert!(!sp.force_sequence_participant_underlined());
}

#[test]
fn force_sequence_participant_underlined_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("sequenceParticipant", "underline");
    assert!(sp.force_sequence_participant_underlined());
}

#[test]
fn default_response_message_below_arrow() {
    let sp = SkinParam::default();
    assert!(!sp.response_message_below_arrow());
}

#[test]
fn response_message_below_arrow_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("responseMessageBelowArrow", "true");
    assert!(sp.response_message_below_arrow());
}

#[test]
fn default_fix_circle_label_overlapping() {
    let sp = SkinParam::default();
    assert!(!sp.fix_circle_label_overlapping());
}

#[test]
fn fix_circle_label_overlapping_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("fixCircleLabelOverlapping", "true");
    assert!(sp.fix_circle_label_overlapping());
}

#[test]
fn default_display_generic_with_old_fashion() {
    let sp = SkinParam::default();
    assert!(!sp.display_generic_with_old_fashion());
}

#[test]
fn display_generic_with_old_fashion_enabled() {
    let mut sp = SkinParam::default();
    sp.set_param("genericDisplay", "old");
    assert!(sp.display_generic_with_old_fashion());
}

#[test]
fn default_shadowing_for_note() {
    let sp = SkinParam::default();
    assert!(sp.shadowing_for_note(None));
}

#[test]
fn note_shadowing_disabled() {
    let mut sp = SkinParam::default();
    sp.set_param("noteshadowing", "false");
    assert!(!sp.shadowing_for_note(None));
}

#[test]
fn strict_uml_component_style() {
    let mut sp = SkinParam::default();
    sp.set_param("style", "strictuml");
    assert_eq!(sp.component_style(), ComponentStyle::Uml2);
}

#[test]
fn custom_component_style() {
    let mut sp = SkinParam::default();
    sp.set_param("componentstyle", "uml1");
    assert_eq!(sp.component_style(), ComponentStyle::Uml1);
}

#[test]
fn custom_package_style() {
    let mut sp = SkinParam::default();
    sp.set_param("packagestyle", "rectangle");
    assert_eq!(sp.package_style(), PackageStyle::Rectangle);
}

#[test]
fn custom_actor_style() {
    let mut sp = SkinParam::default();
    sp.set_param("actorstyle", "awesome");
    assert_eq!(sp.actor_style(), ActorStyle::Awesome);
}

#[test]
fn custom_condition_style() {
    let mut sp = SkinParam::default();
    sp.set_param("conditionstyle", "diamond");
    assert_eq!(sp.get_condition_style(), ConditionStyle::Diamond);
}

#[test]
fn custom_condition_end_style() {
    let mut sp = SkinParam::default();
    sp.set_param("conditionendstyle", "line");
    assert_eq!(sp.get_condition_end_style(), ConditionEndStyle::Line);
}

#[test]
fn custom_length_adjust() {
    let mut sp = SkinParam::default();
    sp.set_param("lengthadjust", "spacingAndGlyphs");
    assert_eq!(sp.get_length_adjust(), LengthAdjust::SpacingAndGlyphs);
}

#[test]
fn custom_group_inheritance() {
    let mut sp = SkinParam::default();
    sp.set_param("groupinheritance", "5");
    assert_eq!(sp.group_inheritance(), 5);
}

#[test]
fn group_inheritance_one_returns_max() {
    let mut sp = SkinParam::default();
    sp.set_param("groupinheritance", "1");
    assert_eq!(sp.group_inheritance(), i32::MAX);
}

#[test]
fn default_diagram_type() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_diagram_type(), DiagramType::Unknown);
}

#[test]
fn custom_diagram_type() {
    let sp = SkinParam::new(DiagramType::Sequence, Pragma::create_empty());
    assert_eq!(sp.get_diagram_type(), DiagramType::Sequence);
}

#[test]
fn default_get_param_same_class_width() {
    let sp = SkinParam::default();
    assert_eq!(sp.get_param_same_class_width(), 0.0);
}

#[test]
fn set_param_same_class_width() {
    let mut sp = SkinParam::default();
    sp.set_param_same_class_width(100.0);
    assert_eq!(sp.get_param_same_class_width(), 100.0);
}

#[test]
fn copy_all_from() {
    let mut sp = SkinParam::default();
    let mut other = HashMap::new();
    other.insert("shadowing".to_string(), "false".to_string());
    sp.copy_all_from(other);
    assert!(!sp.shadowing(None));
}

#[test]
fn default_values() {
    let sp = SkinParam::default();
    let vals = sp.values();
    assert!(vals.is_empty());
}

#[test]
fn set_param_adds_to_values() {
    let mut sp = SkinParam::default();
    sp.set_param("shadowing", "false");
    let vals = sp.values();
    assert_eq!(vals.get("shadowing"), Some(&"false".to_string()));
}
