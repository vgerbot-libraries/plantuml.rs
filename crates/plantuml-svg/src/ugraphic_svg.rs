//! UGraphicSvg — UGraphic implementation for SVG.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/UGraphicSvg.java`

use crate::svg_graphics::SvgGraphics;
use crate::svg_option::SvgOption;
use indexmap::IndexMap;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{UFont, UFontContext};
use plantuml_klimt::changes::{UChangeColor, UChangeFont, UChangeStroke, UClip, UTranslate};
use plantuml_klimt::color::{ColorMapper, HColor};
use plantuml_klimt::shapes::{UEllipse, ULine, UPath, UPolygon, URectangle, UText};
use plantuml_klimt::uchange::UChange;
use plantuml_klimt::ushape::UShape;
use std::any::Any;

/// Change types for SVG rendering.
///
/// Wraps all the concrete UChange types into a single enum so the
/// `UGraphic::Change` associated type can represent any change.
#[derive(Debug, Clone)]
pub enum SvgChange {
    Color(UChangeColor),
    Stroke(UChangeStroke),
    Font(UChangeFont),
    Translate(UTranslate),
    Clip(UClip),
    /// Set both fill and stroke to the same color.
    BackColor(HColor),
}

impl UChange for SvgChange {}

/// UGraphic implementation for SVG output.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/UGraphicSvg.java`
pub struct UGraphicSvg {
    svg: SvgGraphics,
    string_bounder: Box<dyn StringBounder>,
    font: UFont,
    translate: UTranslate,
    color_mapper: ColorMapper,
}

impl UGraphicSvg {
    /// Creates a new `UGraphicSvg` with the given seed, options, and string bounder.
    ///
    /// Ported from: `UGraphicSvg.build(long, SvgOption, StringBounder)`.
    #[must_use]
    pub fn new(seed: i64, option: SvgOption, string_bounder: Box<dyn StringBounder>) -> Self {
        let color_mapper = *option.color_mapper();
        let svg = SvgGraphics::new(seed, option);
        Self {
            svg,
            string_bounder,
            font: UFont::sans_serif(14),
            translate: UTranslate::new(0.0, 0.0),
            color_mapper,
        }
    }

    /// Returns the SVG string.
    #[must_use]
    pub fn create_xml(&mut self) -> String {
        self.svg.create_xml()
    }

    /// Returns the SVG string with indentation.
    #[must_use]
    pub fn create_xml_indented(&mut self, indent: usize) -> String {
        self.svg.create_xml_indented(indent)
    }

    /// Draws a shape by downcasting to the concrete type.
    fn draw_shape(&mut self, shape: &dyn UShape) {
        let any = shape.as_any();
        if let Some(r) = any.downcast_ref::<URectangle>() {
            self.svg.svg_rectangle(
                self.translate.dx() + r.x(),
                self.translate.dy() + r.y(),
                r.width(),
                r.height(),
                r.rx(),
                r.ry(),
                0.0,
            );
        } else if let Some(l) = any.downcast_ref::<ULine>() {
            self.svg.svg_line(
                self.translate.dx() + l.x1(),
                self.translate.dy() + l.y1(),
                self.translate.dx() + l.x2(),
                self.translate.dy() + l.y2(),
                0.0,
            );
        } else if let Some(e) = any.downcast_ref::<UEllipse>() {
            self.svg.svg_ellipse(
                self.translate.dx() + e.x(),
                self.translate.dy() + e.y(),
                e.x_radius(),
                e.y_radius(),
                0.0,
            );
        } else if let Some(p) = any.downcast_ref::<UPolygon>() {
            let mut points = p.flat_points();
            for i in (0..points.len()).step_by(2) {
                points[i] += self.translate.dx();
            }
            for i in (1..points.len()).step_by(2) {
                points[i] += self.translate.dy();
            }
            self.svg.svg_polygon(0.0, &points);
        } else if let Some(_path) = any.downcast_ref::<UPath>() {
            // TODO: implement path rendering
        } else if let Some(t) = any.downcast_ref::<UText>() {
            let attrs = IndexMap::new();
            let style = t.font().style();
            let weight = if style.bold { Some("bold") } else { None };
            let font_style = if style.italic { Some("italic") } else { None };
            self.svg.text(
                t.text(),
                self.translate.dx() + t.x(),
                self.translate.dy() + t.y(),
                Some(t.font().family("", UFontContext::Svg)),
                t.font().size(),
                weight,
                font_style,
                None,
                t.text_length(),
                &attrs,
                None,
            );
        }
    }

    /// Applies a change to the SVG state.
    fn apply_change(&mut self, change: SvgChange) {
        match change {
            SvgChange::Color(c) => {
                let color_str = c.color().to_svg(&self.color_mapper);
                self.svg.set_fill_color(&color_str);
                self.svg.set_stroke_color(Some(&color_str));
            }
            SvgChange::Stroke(s) => {
                self.svg
                    .set_stroke_width(s.stroke_width(), s.dash_array());
            }
            SvgChange::Font(f) => {
                self.font = f.font().clone();
            }
            SvgChange::Translate(t) => {
                self.translate = self.translate.compose(&t);
            }
            SvgChange::Clip(_) => {
                // TODO: implement clipping
            }
            SvgChange::BackColor(c) => {
                let color_str = c.to_svg(&self.color_mapper);
                self.svg.set_fill_color(&color_str);
            }
        }
    }
}

/// Implement the UGraphic trait for UGraphicSvg.
impl plantuml_klimt::ugraphic::UGraphic for UGraphicSvg {
    type Change = SvgChange;

    fn draw(&mut self, shape: &dyn UShape) {
        self.draw_shape(shape);
    }

    fn apply(&mut self, change: Self::Change) {
        self.apply_change(change);
    }

    fn get_string_bounder(&self) -> &dyn StringBounder {
        self.string_bounder.as_ref()
    }

    fn get_font(&self) -> &UFont {
        &self.font
    }
}
