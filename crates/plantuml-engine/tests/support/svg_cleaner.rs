//! SVG normalisation for parity testing.
//!
//! Ported from: `test/vega/SvgCleaner.java`
//!
//! Strips processing instructions and pretty-prints XML so that
//! whitespace differences are eliminated before comparison. Does NOT
//! strip coordinates — comparison is exact after normalisation.

use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;

/// SVG cleaner / normaliser for test comparison.
///
/// Ported from: `test/vega/SvgCleaner.java`
pub struct SvgCleaner;

impl SvgCleaner {
    /// Cleans the raw SVG string: removes processing instructions and
    /// pretty-prints the XML.
    ///
    /// Ported from: `SvgCleaner.clean(String)`.
    pub fn clean(svg_xml: &str) -> String {
        Self::pretty_print(svg_xml, true)
    }

    /// Normalises an SVG string (parse + pretty-print) so that whitespace
    /// differences are eliminated before comparison.
    ///
    /// Ported from: `SvgCleaner.normalise(String)`.
    pub fn normalise(svg_xml: &str) -> String {
        Self::pretty_print(svg_xml, false)
    }

    // ------------------------------------------------------------------

    /// Parse XML events and re-serialize with 2-space indentation.
    ///
    /// When `remove_pi` is true, processing instructions (`<?...?>`) and
    /// the XML declaration are stripped — matching `SvgCleaner.clean()`.
    /// When false, only the XML declaration is stripped (matching
    /// `SvgCleaner.normalise()`, which uses the same transformer that
    /// omits the XML declaration).
    fn pretty_print(svg_xml: &str, remove_pi: bool) -> String {
        let mut reader = Reader::from_str(svg_xml);
        reader.config_mut().trim_text(false);

        let mut writer = Writer::new(Vec::new());
        let mut depth: usize = 0;

        loop {
            let event = match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(event) => event,
                Err(_) => break,
            };

            match event {
                Event::Decl(_) => {
                    // Omit XML declaration (OutputKeys.OMIT_XML_DECLARATION = "yes").
                }
                Event::PI(_) if remove_pi => {
                    // Remove processing instructions (SvgCleaner.clean only).
                }
                Event::PI(pi) => {
                    let _ = writer.write_event(Event::PI(pi));
                }
                Event::Start(start) => {
                    Self::write_indent(&mut writer, depth);
                    let _ = writer.write_event(Event::Start(start));
                    depth += 1;
                }
                Event::End(end) => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    Self::write_indent(&mut writer, depth);
                    let _ = writer.write_event(Event::End(end));
                }
                Event::Empty(empty) => {
                    Self::write_indent(&mut writer, depth);
                    let _ = writer.write_event(Event::Empty(empty));
                }
                Event::Text(text) => {
                    let trimmed = text.unescape().unwrap_or_default();
                    if !trimmed.trim().is_empty() {
                        // Non-whitespace text: write on current line.
                        let _ = writer.write_event(Event::Text(text));
                    }
                    // Whitespace-only text between elements is stripped
                    // (XSL: strip-space elements="*").
                }
                Event::CData(cdata) => {
                    let _ = writer.write_event(Event::CData(cdata));
                }
                Event::Comment(comment) => {
                    Self::write_indent(&mut writer, depth);
                    let _ = writer.write_event(Event::Comment(comment));
                }
                Event::DocType(_) | Event::Eof => {}
            }
        }

        String::from_utf8(writer.into_inner()).unwrap_or_default()
    }

    /// Writes a newline + 2-space-per-depth indentation.
    fn write_indent(writer: &mut Writer<Vec<u8>>, depth: usize) {
        writer.get_mut().push(b'\n');
        for _ in 0..depth {
            writer.get_mut().extend_from_slice(b"  ");
        }
    }
}
