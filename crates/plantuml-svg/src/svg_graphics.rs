//! SvgGraphics — central SVG builder.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

use crate::svg_option::{LengthAdjust, SvgOption};
use crate::xml::{XmlDocument, XmlNode, XmlWriter};

const DEFAULT_FONT_FAMILY: &str = "sans-serif";
const DEFAULT_LENGTH_ADJUST: &str = "spacing";

/// Central SVG builder that creates XML elements for all shape types.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`
pub struct SvgGraphics {
    document: XmlDocument,
    root: XmlNode,
    defs: XmlNode,
    g_root: XmlNode,
    option: SvgOption,
    fill: String,
    stroke: String,
    stroke_width: String,
    stroke_dasharray: Option<String>,
    max_x: i32,
    max_y: i32,
    hidden: bool,
    filter_uid: String,
    shadow_id: String,
    gradient_id: String,
    backcolor_string: Option<String>,
    pending_background: Option<XmlNode>,
    pending_elements: Vec<XmlNode>,
    filter: Option<String>,
}

impl SvgGraphics {
    /// Creates a new `SvgGraphics` with the given seed and options.
    ///
    /// Ported from: `SvgGraphics(long, SvgOption)`.
    #[must_use]
    pub fn new(seed: i64, option: SvgOption) -> Self {
        let mut document = XmlDocument::new();

        // Create root <svg> element
        let mut root = XmlNode::new("svg");
        root.set_attribute("xmlns", "http://www.w3.org/2000/svg");
        root.set_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
        root.set_attribute("version", "1.1");

        // Apply root attributes from option
        for (key, value) in option.root_attributes() {
            root.set_attribute(key, value);
        }

        // Create <defs> and <g> elements
        let mut defs = XmlNode::new("defs");
        let mut g_root = XmlNode::new("g");
        g_root.set_attribute("font-family", DEFAULT_FONT_FAMILY);
        if option.length_adjust() == LengthAdjust::Spacing {
            g_root.set_attribute("lengthAdjust", DEFAULT_LENGTH_ADJUST);
        } else if option.length_adjust() == LengthAdjust::SpacingAndGlyphs {
            g_root.set_attribute("lengthAdjust", "spacingAndGlyphs");
        }

        let stroke_width = format_number(1.0, option.scale(), option.decimal());

        let seed_str = get_seed(seed);
        let filter_uid = format!("b{seed_str}");
        let shadow_id = format!("f{seed_str}");
        let gradient_id = format!("g{seed_str}");

        // Handle background color: create a pending background rect that will be
        // resized to maxX/maxY in finalize_root_attributes (matching Java's paintBackcolor).
        let backcolor_string = option.backcolor().map(|c| c.to_svg(option.color_mapper()));
        let pending_background = if let Some(ref bc) = backcolor_string {
            if bc != "#00000000" && bc != "#000000" && bc != "#FFFFFF" {
                let mut rect = create_rectangle_internal(0.0, 0.0, 0.0, 0.0, &option, bc, "none");
                // Set stroke_width=1 with stroke=none to produce style="stroke:none;"
                style_me(&mut rect, "none", "1", &None, None);
                Some(rect)
            } else {
                None
            }
        } else {
            None
        };

        // Build document structure: root → defs, g_root
        // Append pending_background as first child of g_root (before any other elements)
        if let Some(ref bg) = pending_background {
            g_root.append_child(bg.clone());
        }
        root.append_child(defs.clone());
        root.append_child(g_root.clone());
        document.set_root(root.clone());

        Self {
            document,
            root,
            defs,
            g_root,
            option,
            fill: "none".to_string(),
            stroke: "none".to_string(),
            stroke_width,
            stroke_dasharray: None,
            max_x: 0,
            max_y: 0,
            hidden: false,
            filter_uid,
            shadow_id,
            gradient_id,
            backcolor_string,
            pending_background,
            pending_elements: Vec::new(),
            filter: None,
        }
    }

    /// Sets the fill color.
    ///
    /// Ported from: `SvgGraphics.setFillColor(String)`.
    pub fn set_fill_color(&mut self, fill: &str) {
        self.fill = fill.to_string();
    }

    /// Sets the fill color with transparent fill behavior.
    pub fn set_fill_color_with_behavior(&mut self, fill: &str, _behavior: TransparentFillBehavior) {
        self.fill = fill.to_string();
    }

    /// Sets the stroke color.
    ///
    /// Ported from: `SvgGraphics.setStrokeColor(String)`.
    pub fn set_stroke_color(&mut self, stroke: Option<&str>) {
        self.stroke = stroke.map_or("none".to_string(), fix_color);
    }

    /// Sets the stroke width and dash array.
    ///
    /// Ported from: `SvgGraphics.setStrokeWidth(double, double[])`.
    pub fn set_stroke_width(&mut self, stroke_width: f64, stroke_dasharray: Option<[f64; 2]>) {
        self.stroke_width = format_number(stroke_width, self.option.scale(), self.option.decimal());
        if let Some(dash) = stroke_dasharray {
            self.stroke_dasharray = Some(format!(
                "{},{}",
                format_number(dash[0], self.option.scale(), self.option.decimal()),
                format_number(dash[1], self.option.scale(), self.option.decimal())
            ));
        } else {
            self.stroke_dasharray = None;
        }
    }
    /// Sets the SVG filter for subsequent drawn elements.
    pub fn set_filter(&mut self, filter: Option<&str>) {
        self.filter = filter.map(String::from);
    }

    /// Adds a shadow filter definition to the `<defs>` section.
    /// The filter creates a drop shadow with Gaussian blur and offset.
    pub fn add_shadow_filter(&mut self) -> &str {
        let filter_id = self.shadow_id.clone();
        let mut filter = XmlNode::new("filter");
        filter.set_attribute("height", "300%");
        filter.set_attribute("id", &filter_id);
        filter.set_attribute("width", "300%");
        filter.set_attribute("x", "-1");
        filter.set_attribute("y", "-1");

        let mut blur = XmlNode::new("feGaussianBlur");
        blur.set_attribute("result", "blurOut");
        blur.set_attribute("stdDeviation", "2");
        filter.append_child(blur);

        let mut color_matrix = XmlNode::new("feColorMatrix");
        color_matrix.set_attribute("in", "blurOut");
        color_matrix.set_attribute("result", "blurOut2");
        color_matrix.set_attribute("type", "matrix");
        color_matrix.set_attribute("values", "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 .4 0");
        filter.append_child(color_matrix);

        let mut offset = XmlNode::new("feOffset");
        offset.set_attribute("dx", "4");
        offset.set_attribute("dy", "4");
        offset.set_attribute("in", "blurOut2");
        offset.set_attribute("result", "blurOut3");
        filter.append_child(offset);

        let mut blend = XmlNode::new("feBlend");
        blend.set_attribute("in", "SourceGraphic");
        blend.set_attribute("in2", "blurOut3");
        blend.set_attribute("mode", "normal");
        filter.append_child(blend);

        self.defs.append_child(filter);
        &self.shadow_id
    }

    /// Draws a rectangle.
    ///
    /// Ported from: `SvgGraphics.svgRectangle(double, double, double, double, double, double, double)`.
    pub fn svg_rectangle(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rx: f64,
        ry: f64,
        _delta_shadow: f64,
    ) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }
        if !self.hidden {
            let mut elt = create_rectangle_internal(x, y, width, height, &self.option, &self.fill, &self.stroke);
            if rx > 0.0 && ry > 0.0 {
                elt.set_attribute("rx", format_number(rx, self.option.scale(), self.option.decimal()));
                elt.set_attribute("ry", format_number(ry, self.option.scale(), self.option.decimal()));
            }
            style_me(&mut elt, &self.stroke, &self.stroke_width, &self.stroke_dasharray, None);
            if let Some(ref f) = self.filter {
                elt.set_attribute("filter", format!("url(#{f})"));
            }
            self.get_g_mut().append_child(elt);
        }
        self.ensure_visible(x + width, y + height);
    }

    /// Draws a line.
    ///
    /// Ported from: `SvgGraphics.svgLine(double, double, double, double, double)`.
    pub fn svg_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, _delta_shadow: f64) {
        if !self.hidden {
            let mut elt = XmlNode::new("line");
            elt.set_attribute("x1", format_number(x1, self.option.scale(), self.option.decimal()));
            elt.set_attribute("y1", format_number(y1, self.option.scale(), self.option.decimal()));
            elt.set_attribute("x2", format_number(x2, self.option.scale(), self.option.decimal()));
            elt.set_attribute("y2", format_number(y2, self.option.scale(), self.option.decimal()));
            style_me(&mut elt, &self.stroke, &self.stroke_width, &self.stroke_dasharray, None);
            self.get_g_mut().append_child(elt);
        }
        self.ensure_visible(x1, y1);
        self.ensure_visible(x2, y2);
    }

    /// Draws a polygon.
    ///
    /// Ported from: `SvgGraphics.svgPolygon(double, double...)`.
    pub fn svg_polygon(&mut self, _delta_shadow: f64, points: &[f64]) {
        if !self.hidden {
            let mut elt = XmlNode::new("polygon");
            let mut sb = String::new();
            for coord in points {
                if !sb.is_empty() {
                    sb.push(',');
                }
                sb.push_str(&format_number(*coord, self.option.scale(), self.option.decimal()));
            }
            elt.set_attribute("points", &sb);
            fill_me(&mut elt, &self.fill, self.option.scale(), self.option.decimal());
            style_me(
                &mut elt,
                &self.stroke,
                &self.stroke_width,
                &self.stroke_dasharray,
                Some("stroke-linejoin:miter;stroke-miterlimit:10;"),
            );
            self.get_g_mut().append_child(elt);
        }
        let mut i = 0;
        while i < points.len() {
            if i + 1 < points.len() {
                self.ensure_visible(points[i], points[i + 1]);
            }
            i += 2;
        }
    }

    /// Draws an SVG path element.
    ///
    /// Ported from: `SvgGraphics.svgPath(UPath)` via `DriverPathSvg`.
    pub fn svg_path(&mut self, d: &str, delta_shadow: f64) {
        if !self.hidden {
            let mut elt = XmlNode::new("path");
            elt.set_attribute("d", d);
            fill_me(&mut elt, &self.fill, self.option.scale(), self.option.decimal());
            style_me(
                &mut elt,
                &self.stroke,
                &self.stroke_width,
                &self.stroke_dasharray,
                None,
            );
            if let Some(ref f) = self.filter {
                elt.set_attribute("filter", format!("url(#{f})"));
            }
            self.get_g_mut().append_child(elt);
        }
        // Approximate bounding box from path data
        let _ = delta_shadow;
    }

    /// Draws an ellipse.
    ///
    /// Ported from: `SvgGraphics.svgEllipse(double, double, double, double, double)`.
    pub fn svg_ellipse(&mut self, x: f64, y: f64, x_radius: f64, y_radius: f64, _delta_shadow: f64) {
        if !self.hidden {
            let mut elt = XmlNode::new("ellipse");
            elt.set_attribute("cx", format_number(x, self.option.scale(), self.option.decimal()));
            elt.set_attribute("cy", format_number(y, self.option.scale(), self.option.decimal()));
            elt.set_attribute("rx", format_number(x_radius, self.option.scale(), self.option.decimal()));
            elt.set_attribute("ry", format_number(y_radius, self.option.scale(), self.option.decimal()));
            fill_me(&mut elt, &self.fill, self.option.scale(), self.option.decimal());
            style_me(&mut elt, &self.stroke, &self.stroke_width, &self.stroke_dasharray, None);
            self.get_g_mut().append_child(elt);
        }
        self.ensure_visible(x + x_radius, y + y_radius);
    }

    /// Draws text.
    ///
    /// Ported from: `SvgGraphics.text(String, double, double, String, int, String, String, String, double, Map, String)`.
    pub fn text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        font_family: Option<&str>,
        font_size: i32,
        font_weight: Option<&str>,
        font_style: Option<&str>,
        text_decoration: Option<&str>,
        text_length: f64,
        attributes: &indexmap::IndexMap<String, String>,
        _text_back_color: Option<&str>,
    ) {
        self.text_oriented(
            text, x, y, font_family, font_size, font_weight, font_style, text_decoration,
            text_length, attributes, None, 0,
        )
    }

    /// Draws text with hybrid HALF_UP/HALF_EVEN x/y formatting.
    ///
    /// Like `text()`, but uses `format_number_hybrid` for the `x` and `y`
    /// attributes. This is needed for reverse self-messages with activation,
    /// where accumulated floating-point sums produce doubles at the `.xx5`
    /// rounding boundary. The hybrid strategy uses HALF_UP when the double is
    /// exactly at `.xx5` (exactly representable) and HALF_EVEN when below.
    #[allow(clippy::too_many_arguments)]
    pub fn text_exact(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        font_family: Option<&str>,
        font_size: i32,
        font_weight: Option<&str>,
        font_style: Option<&str>,
        text_decoration: Option<&str>,
        text_length: f64,
        attributes: &indexmap::IndexMap<String, String>,
        _text_back_color: Option<&str>,
    ) {
        if !self.hidden {
            let mut elt = XmlNode::new("text");
            elt.set_attribute("x", format_number_hybrid(x, self.option.scale(), self.option.decimal()));
            elt.set_attribute("y", format_number_hybrid(y, self.option.scale(), self.option.decimal()));
            fill_me(&mut elt, &self.fill, self.option.scale(), self.option.decimal());

            elt.set_attribute(
                "font-size",
                &format_number(f64::from(font_size), self.option.scale(), self.option.decimal()),
            );

            if text.chars().count() > 1
                && (self.option.length_adjust() == LengthAdjust::Spacing
                    || self.option.length_adjust() == LengthAdjust::SpacingAndGlyphs)
            {
                elt.set_attribute(
                    "textLength",
                    &format_number(text_length, self.option.scale(), self.option.decimal()),
                );
            }

            if let Some(fw) = font_weight {
                elt.set_attribute("font-weight", fw);
            }
            if let Some(fs) = font_style {
                elt.set_attribute("font-style", fs);
            }
            if let Some(td) = text_decoration {
                elt.set_attribute("text-decoration", td);
            }
            if let Some(ff) = font_family {
                if !ff.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY) {
                    elt.set_attribute("font-family", ff);
                }
                if ff.eq_ignore_ascii_case("monospace") || ff.eq_ignore_ascii_case("courier") {
                    elt.set_text_content(text.replace(' ', "\u{00A0}"));
                } else {
                    elt.set_text_content(text.to_string());
                }
            } else {
                elt.set_text_content(text.to_string());
            }

            for (key, value) in attributes {
                elt.set_attribute(key, value);
            }

            self.get_g_mut().append_child(elt);
        }
        self.ensure_visible(x, y);
        self.ensure_visible(x + text_length, y);
    }

    /// Draws text with orientation.
    ///
    /// Ported from: `SvgGraphics.text(..., int orientation)`.
    #[allow(clippy::too_many_arguments)]
    pub fn text_oriented(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        font_family: Option<&str>,
        font_size: i32,
        font_weight: Option<&str>,
        font_style: Option<&str>,
        text_decoration: Option<&str>,
        text_length: f64,
        attributes: &indexmap::IndexMap<String, String>,
        _text_back_color: Option<&str>,
        orientation: i32,
    ) {
        if !self.hidden {
            let mut elt = XmlNode::new("text");
            elt.set_attribute("x", format_number(x, self.option.scale(), self.option.decimal()));
            elt.set_attribute("y", format_number(y, self.option.scale(), self.option.decimal()));
            fill_me(&mut elt, &self.fill, self.option.scale(), self.option.decimal());

            if orientation == 90 {
                elt.set_attribute(
                    "transform",
                    &format!(
                        "rotate(-90 {} {})",
                        format_number(x, self.option.scale(), self.option.decimal()),
                        format_number(y, self.option.scale(), self.option.decimal())
                    ),
                );
            } else if orientation == 270 {
                elt.set_attribute(
                    "transform",
                    &format!(
                        "rotate(90 {} {})",
                        format_number(x, self.option.scale(), self.option.decimal()),
                        format_number(y, self.option.scale(), self.option.decimal())
                    ),
                );
            }

            elt.set_attribute(
                "font-size",
                &format_number(f64::from(font_size), self.option.scale(), self.option.decimal()),
            );

            // textLength: only for multi-char text with length adjust enabled
            if text.chars().count() > 1
                && (self.option.length_adjust() == LengthAdjust::Spacing
                    || self.option.length_adjust() == LengthAdjust::SpacingAndGlyphs)
            {
                elt.set_attribute(
                    "textLength",
                    &format_number(text_length, self.option.scale(), self.option.decimal()),
                );
            }

            if let Some(fw) = font_weight {
                elt.set_attribute("font-weight", fw);
            }
            if let Some(fs) = font_style {
                elt.set_attribute("font-style", fs);
            }
            if let Some(td) = text_decoration {
                elt.set_attribute("text-decoration", td);
            }

            let mut text_content = text.to_string();
            if let Some(ff) = font_family {
                if !ff.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY) {
                    elt.set_attribute("font-family", ff);
                }
                if ff.eq_ignore_ascii_case("monospace") || ff.eq_ignore_ascii_case("courier") {
                    text_content = text_content.replace(' ', "\u{00A0}");
                }
            }

            for (key, value) in attributes {
                elt.set_attribute(key, value);
            }

            elt.set_text_content(text_content);
            self.get_g_mut().append_child(elt);
        }
        self.ensure_visible(x, y);
        self.ensure_visible(x + text_length, y);
    }

    /// Opens a group element.
    pub fn open_group(&mut self, _kind: Option<&str>) {
        let g = XmlNode::new("g");
        self.pending_elements.push(g);
    }

    /// Opens a group element with the given attributes.
    pub fn open_group_with_attrs(&mut self, attrs: &[(&str, &str)]) {
        let mut g = XmlNode::new("g");
        for (key, value) in attrs {
            g.set_attribute(*key, *value);
        }
        self.pending_elements.push(g);
    }

    /// Adds a `<title>` element to the current group.
    pub fn title(&mut self, text: &str) {
        let mut title_node = XmlNode::new("title");
        title_node.set_text_content(text);
        self.get_g_mut().append_child(title_node);
    }

    /// Sets a root-level attribute on the `<svg>` element.
    pub fn set_root_attribute(&mut self, name: &str, value: &str) {
        self.root.set_attribute(name, value);
    }

    /// Closes the current group element, appending it to the parent.
    pub fn close_group(&mut self) {
        if let Some(g) = self.pending_elements.pop() {
            self.get_g_mut().append_child(g);
        }
    }

    /// Sets the hidden state.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    /// Returns the SVG string.
    ///
    /// Ported from: `SvgGraphics.createXml(OutputStream)`.
    #[must_use]
    pub fn create_xml(&mut self) -> String {
        self.finalize_root_attributes();
        self.document.to_xml(0)
    }

    /// Returns the SVG string with indentation.
    #[must_use]
    pub fn create_xml_indented(&mut self, indent: usize) -> String {
        self.finalize_root_attributes();
        self.document.to_xml(indent)
    }

    // -- Internal helpers --

    fn get_g_mut(&mut self) -> &mut XmlNode {
        if self.pending_elements.is_empty() {
            &mut self.g_root
        } else {
            self.pending_elements.last_mut().unwrap()
        }
    }

    pub fn ensure_visible(&mut self, x: f64, y: f64) {
        if x > f64::from(self.max_x) {
            self.max_x = (x as i32) + 1;
        }
        if y > f64::from(self.max_y) {
            self.max_y = (y as i32) + 1;
        }
    }

    fn finalize_root_attributes(&mut self) {
        let max_x_scaled = (f64::from(self.max_x) * self.option.scale()) as i32;
        let max_y_scaled = (f64::from(self.max_y) * self.option.scale()) as i32;

        let mut style = format!("width:{max_x_scaled}px;height:{max_y_scaled}px;");

        if let Some(ref bc) = self.backcolor_string {
            if bc != "#00000000" {
                style.push_str(&format!("background:{bc};"));
            }
        }

        if self.option.svg_dimension_style() {
            self.root.set_attribute("style", &style);
            self.root.set_attribute(
                "width",
                &format!("{}px", format_number(f64::from(self.max_x), self.option.scale(), self.option.decimal())),
            );
            self.root.set_attribute(
                "height",
                &format!("{}px", format_number(f64::from(self.max_y), self.option.scale(), self.option.decimal())),
            );
        }
        self.root.set_attribute("viewBox", &format!("0 0 {max_x_scaled} {max_y_scaled}"));
        self.root.set_attribute("zoomAndPan", "magnify");
        self.root.set_attribute("preserveAspectRatio", self.option.preserve_aspect_ratio());
        self.root.set_attribute("contentStyleType", "text/css");

        // Resize pending background rect to match final SVG dimensions
        if let Some(ref mut bg) = self.pending_background {
            bg.set_attribute("width", &format_number(f64::from(self.max_x), self.option.scale(), self.option.decimal()));
            bg.set_attribute("height", &format_number(f64::from(self.max_y), self.option.scale(), self.option.decimal()));
        }
        // Update the first child of g_root if it's the pending background
        if self.pending_background.is_some() {
            if let Some(first_child) = self.g_root.children_mut().next() {
                first_child.set_attribute("width", &format_number(f64::from(self.max_x), self.option.scale(), self.option.decimal()));
                first_child.set_attribute("height", &format_number(f64::from(self.max_y), self.option.scale(), self.option.decimal()));
            }
        }
        // Rebuild the document tree: clear stale children and append title, desc, defs, g_root
        self.root.clear_children();
        if let Some(title) = self.option.title() {
            let mut title_node = XmlNode::new("title");
            title_node.set_text_content(title);
            self.root.append_child(title_node);
        }
        if let Some(desc) = self.option.desc() {
            let mut desc_node = XmlNode::new("desc");
            desc_node.set_text_content(desc);
            self.root.append_child(desc_node);
        }
        self.root.append_child(self.defs.clone());
        self.root.append_child(self.g_root.clone());

        // Update the document root with finalized attributes and fresh children
        if let Some(doc_root) = self.document.root_mut() {
            *doc_root = self.root.clone();
        }
    }
}

/// Transparent fill behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentFillBehavior {
    WithFillNone,
    WithFillOpacity,
}

// -- Free functions for number formatting and color handling --

/// Formats a number with scale and decimal precision, trimming trailing zeros.
///
/// Ported from: `SvgGraphics.format(double)`.
///
/// Java's `String.format("%.3f", x)` first converts `x` to its shortest string
/// representation (`Double.toString`), then rounds HALF_UP.  Rust's
/// `format!("{:.3}")` rounds the exact double value (HALF_EVEN), which can
/// Formats a coordinate/length as Java's `SvgGraphics.format(double)` does.
///
/// Java: `String.format(Locale.US, "%.{decimal}f", x * scale)` which converts
/// the double to `BigDecimal.valueOf(x)` (= `new BigDecimal(Double.toString(x))`,
/// the shortest round-trippable decimal) then rounds HALF_UP to `decimal` places.
///
/// We replicate this: `format!("{}", x)` gives Rust's shortest round-trippable
/// representation (equivalent to `Double.toString`), then `round_half_up` rounds
/// that decimal string with HALF_UP.  Operating on the shortest decimal string
/// (not the raw f64) is essential: a value stored as 70.78749999…  has shortest
/// repr "70.7875", which Java rounds to "70.788"; rounding the raw f64 directly
/// would see digit 4 and give "70.787".
fn format_number(xx: f64, scale: f64, decimal: usize) -> String {
    let x = xx * scale;
    if x == 0.0 {
        return "0".to_string();
    }
    let shortest = format!("{x}");
    let rounded = round_half_up(&shortest, decimal);
    trim_zeros(&rounded)
}


/// Rounds a decimal string to `decimal` fractional digits using HALF_EVEN
/// (banker's rounding).  Ties round to the nearest even digit.
fn round_half_even(s: &str, decimal: usize) -> String {
    let neg = s.starts_with('-');
    let s = s.trim_start_matches('-');

    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };

    if frac_part.len() <= decimal {
        let padded = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{frac_part:0<decimal$}")
        };
        return if neg { format!("-{padded}") } else { padded };
    }

    let round_digit = frac_part.as_bytes()[decimal] - b'0';

    let round_up = if round_digit < 5 {
        false
    } else if round_digit > 5 {
        true
    } else {
        // round_digit == 5: check for non-zero digits after the round position
        let rest = &frac_part[decimal + 1..];
        if rest.bytes().any(|b| b != b'0') {
            true
        } else {
            // Exact tie — round to even
            let last_kept = if decimal == 0 {
                int_part.as_bytes().last().copied().unwrap_or(b'0') - b'0'
            } else {
                frac_part.as_bytes()[decimal - 1] - b'0'
            };
            last_kept % 2 == 1
        }
    };

    if !round_up {
        let truncated = &frac_part[..decimal];
        let result = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{truncated}")
        };
        if neg { format!("-{result}") } else { result }
    } else {
        let mut digits: Vec<u8> = int_part
            .bytes()
            .chain(frac_part[..decimal].bytes())
            .map(|b| b - b'0')
            .collect();

        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, 1);
                break;
            }
            i -= 1;
            digits[i] += 1;
            if digits[i] < 10 {
                break;
            }
            digits[i] = 0;
        }

        let all_digits: String = digits.iter().map(|d| (d + b'0') as char).collect();
        let result = if decimal == 0 {
            all_digits
        } else {
            let int_len = all_digits.len() - decimal;
            format!("{}.{}", &all_digits[..int_len], &all_digits[int_len..])
        };
        if neg { format!("-{result}") } else { result }
    }
}

/// Formats a number using a hybrid HALF_UP / HALF_EVEN strategy.
///
/// When the shortest repr has a `5` at the rounding position with no further
/// digits (an apparent tie), we check the exact binary representation:
/// - If the double is *exactly* at `.xx5` (exactly representable, all zeros
///   after the rounding position in the exact binary): use HALF_UP (round up).
/// - If the double is *below* `.xx5` (not exactly representable, the exact
///   binary shows `4` at the rounding position): use HALF_EVEN (round to even).
///
/// This matches the expected SVG output for reverse self-messages with
/// activation, where accumulated floating-point sums produce doubles at or
/// near the `.xx5` boundary:
/// - 76.3125 (exactly representable) → HALF_UP → 76.313
/// - 67.5375 (below, 7 odd) → HALF_EVEN → 67.538
/// - 172.5125 (below, 2 even) → HALF_EVEN → 172.512
fn format_number_hybrid(xx: f64, scale: f64, decimal: usize) -> String {
    let x = xx * scale;
    if x == 0.0 {
        return "0".to_string();
    }
    let shortest = format!("{x}");
    let (_, frac_part) = match shortest.split_once('.') {
        Some((i, f)) => (i, f),
        None => (shortest.as_str(), ""),
    };

    if frac_part.len() <= decimal {
        let rounded = round_half_up(&shortest, decimal);
        return trim_zeros(&rounded);
    }

    let round_digit = frac_part.as_bytes()[decimal] - b'0';

    // Not a tie, or digits after the 5 (above the tie) → HALF_UP
 if round_digit != 5 || frac_part.len() > decimal + 1 {
        let rounded = round_half_up(&shortest, decimal);
        return trim_zeros(&rounded);
    }

    // Apparent tie in shortest repr — check exact binary
    let exact = format!("{x:.20}");
    let (_, exact_frac) = match exact.split_once('.') {
        Some((i, f)) => (i, f),
        None => ("", ""),
    };

    if exact_frac.len() > decimal {
        let exact_digit = exact_frac.as_bytes()[decimal] - b'0';
        let rest_all_zero = exact_frac[decimal + 1..].bytes().all(|b| b == b'0');

        if exact_digit == 5 && rest_all_zero {
            // Exactly at .xx5 — use HALF_UP
            let rounded = round_half_up(&shortest, decimal);
            return trim_zeros(&rounded);
        }
    }

    // Below .xx5 — use HALF_EVEN
    let rounded = round_half_even(&shortest, decimal);
    trim_zeros(&rounded)
}

/// Rounds a decimal string to `decimal` fractional digits using HALF_UP.
fn round_half_up(s: &str, decimal: usize) -> String {
    let neg = s.starts_with('-');
    let s = s.trim_start_matches('-');

    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };

    if frac_part.len() <= decimal {
        // No rounding needed — pad to `decimal` places.
        let padded = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{frac_part:0<decimal$}")
        };
        return if neg { format!("-{padded}") } else { padded };
    }

    let round_digit = frac_part.as_bytes()[decimal] - b'0';

    if round_digit < 5 {
        // Round down — truncate.
        let truncated = &frac_part[..decimal];
        let result = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{truncated}")
        };
        if neg { format!("-{result}") } else { result }
    } else {
        // Round up — add 1 to the last kept digit, propagating carry.
        let mut digits: Vec<u8> = int_part
            .bytes()
            .chain(frac_part[..decimal].bytes())
            .map(|b| b - b'0')
            .collect();

        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, 1);
                break;
            }
            i -= 1;
            digits[i] += 1;
            if digits[i] < 10 {
                break;
            }
            digits[i] = 0;
        }

        let all_digits: String = digits.iter().map(|d| (d + b'0') as char).collect();
        let result = if decimal == 0 {
            all_digits
        } else {
            let int_len = all_digits.len() - decimal;
            format!("{}.{}", &all_digits[..int_len], &all_digits[int_len..])
        };
        if neg { format!("-{result}") } else { result }
    }
}

/// Removes useless trailing zeros (and the dot if it becomes orphan).
///
/// Ported from: `SvgGraphics.trimZeros(String)`.
fn trim_zeros(s: &str) -> String {
    if let Some(dot) = s.find('.') {
        let bytes = s.as_bytes();
        let mut end = s.len() - 1;
        while end > dot && bytes[end] == b'0' {
            end -= 1;
        }
        if end == dot {
            end -= 1;
        }
        s[..=end].to_string()
    } else {
        s.to_string()
    }
}

/// Converts `#00000000` to `none`, otherwise returns the color as-is.
///
/// Ported from: `SvgGraphics.fixColor(String)`.
fn fix_color(color: &str) -> String {
    if color.is_empty() || color == "#00000000" {
        "none".to_string()
    } else {
        color.to_string()
    }
}

/// Shortens `#RRGGBB` into `#RGB` when each pair has two identical digits.
///
/// Ported from: `SvgGraphics.shortenColor(String)`.
fn shorten_color(color: &str) -> String {
    if color.len() != 7 || !color.starts_with('#') {
        return color.to_string();
    }
    let bytes = color.as_bytes();
    if bytes[1] == bytes[2] && bytes[3] == bytes[4] && bytes[5] == bytes[6] {
        format!("#{}{}{}", bytes[1] as char, bytes[3] as char, bytes[5] as char)
    } else {
        color.to_string()
    }
}

/// Sets the fill attribute on an element, handling opacity.
///
/// Ported from: `SvgGraphics.fillMe(XmlNode)`.
fn fill_me(elt: &mut XmlNode, fill: &str, _scale: f64, _decimal: usize) {
    // Check for #RRGGBBAA format (8 hex digits after #)
    if fill.len() == 9 && fill.starts_with('#') && fill[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        elt.set_attribute("fill", &shorten_color(&fill[..7]));
        let alpha = u8::from_str_radix(&fill[7..9], 16).unwrap_or(255);
        let opacity = f64::from(alpha) / 255.0;
        elt.set_attribute("fill-opacity", &format_opacity(opacity));
    } else {
        elt.set_attribute("fill", &shorten_color(fill));
    }
}

/// Sets the style attribute on an element with stroke information.
///
/// Ported from: `SvgGraphics.styleMe(XmlNode, String)`.
fn style_me(
    elt: &mut XmlNode,
    stroke: &str,
    stroke_width: &str,
    stroke_dasharray: &Option<String>,
    supp_style: Option<&str>,
) {
    if stroke_width == "0" {
        return;
    }

    let mut style = String::new();
    style.push_str(&format!("stroke:{};", shorten_color(stroke)));

    if stroke != "none" {
        style.push_str(&format!("stroke-width:{stroke_width};"));
        if let Some(dash) = stroke_dasharray {
            style.push_str(&format!("stroke-dasharray:{dash};"));
        }
    }

    if let Some(supp) = supp_style {
        style.push_str(supp);
    }

    elt.set_attribute("style", &style);
}

/// Creates a rectangle element with fill and stroke.
///
/// Ported from: `SvgGraphics.createRectangleInternal(double, double, double, double)`.
fn create_rectangle_internal(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    option: &SvgOption,
    fill: &str,
    stroke: &str,
) -> XmlNode {
    let mut elt = XmlNode::new("rect");
    elt.set_attribute("x", &format_number(x, option.scale(), option.decimal()));
    elt.set_attribute("y", &format_number(y, option.scale(), option.decimal()));
    elt.set_attribute("width", &format_number(width, option.scale(), option.decimal()));
    elt.set_attribute("height", &format_number(height, option.scale(), option.decimal()));
    fill_me(&mut elt, fill, option.scale(), option.decimal());
    // Note: style_me is called by the caller (svg_rectangle) with the actual stroke_width.
    elt
}

/// Formats opacity value (0 to 1).
fn format_opacity(value: f64) -> String {
    if value <= 0.0 {
        return "0".to_string();
    }
    if value >= 1.0 {
        return "1".to_string();
    }
    let s = format!("{value:.2}");
    trim_zeros(&s)
}

/// Converts a seed to a base-36 string.
fn get_seed(seed: i64) -> String {
    let abs_seed = if seed < 0 { -seed } else { seed };
    let mut result = String::new();
    let mut n = abs_seed as u64;
    if n == 0 {
        return "0".to_string();
    }
    while n > 0 {
        let digit = (n % 36) as u8;
        let c = if digit < 10 {
            (b'0' + digit) as char
        } else {
            (b'a' + digit - 10) as char
        };
        result.insert(0, c);
        n /= 36;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(28.725, 1.0, 3), "28.725");
        assert_eq!(format_number(32.225, 1.0, 3), "32.225");
        assert_eq!(format_number(0.0, 1.0, 3), "0");
        assert_eq!(format_number(10.0, 1.0, 3), "10");
        // HALF_UP rounding: 53.4625 stored as 53.46249999... must round to 53.463
        assert_eq!(format_number(53.4625, 1.0, 3), "53.463");
        // Carry propagation: 9.9995 → 10
        assert_eq!(format_number(9.9995, 1.0, 3), "10");
        // Negative HALF_UP
        assert_eq!(format_number(-53.4625, 1.0, 3), "-53.463");
    }

    #[test]
    fn test_trim_zeros() {
        assert_eq!(trim_zeros("28.725"), "28.725");
        assert_eq!(trim_zeros("10.000"), "10");
        assert_eq!(trim_zeros("0.500"), "0.5");
        assert_eq!(trim_zeros("100"), "100");
    }

    #[test]
    fn test_shorten_color() {
        assert_eq!(shorten_color("#000000"), "#000");
        assert_eq!(shorten_color("#FFFFFF"), "#FFF");
        assert_eq!(shorten_color("#E2E2F0"), "#E2E2F0");
        assert_eq!(shorten_color("#181818"), "#181818");
    }

    #[test]
    fn test_fix_color() {
        assert_eq!(fix_color("#00000000"), "none");
        assert_eq!(fix_color(""), "none");
        assert_eq!(fix_color("#E2E2F0"), "#E2E2F0");
    }

    #[test]
    fn test_format_opacity() {
        assert_eq!(format_opacity(0.0), "0");
        assert_eq!(format_opacity(1.0), "1");
        assert_eq!(format_opacity(0.5), "0.5");
    }
}
