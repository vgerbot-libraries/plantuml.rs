//! Deterministic string bounder using the hardcoded Unicode width table.
//!
//! Ported from: net/sourceforge/plantuml/klimt/drawing/font/StringBounderFromWidthTable.java

use plantuml_core::file_format::FileFormat;
use plantuml_core::geom::XDimension2D;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{UFont, UFontContext};

use crate::unicode_block::UnicodeBlock;
use crate::unicode_font_width_sans_serif::SANS_SERIF;

/// Reference font size for the width table (16 pt).
const REFERENCE_SIZE: f64 = 16.0;

/// A `StringBounder` that measures text using the hardcoded Unicode width
/// table. Produces deterministic, platform-independent dimensions — the
/// primary bounder for `SVG_DETERMINISTIC` and `LATEX_DETERMINISTIC` output.
pub struct StringBounderFromWidthTable {
    file_format: FileFormat,
    blocks: Vec<UnicodeBlock>,
}

impl StringBounderFromWidthTable {
    #[must_use]
    pub fn new(file_format: FileFormat) -> Self {
        let blocks = SANS_SERIF
            .iter()
            .map(|raw| UnicodeBlock::new(raw))
            .collect();
        Self { file_format, blocks }
    }

    fn get_char_width(&self, cp: u32) -> f64 {
        if cp >= 0xFFFF {
            return 16.0;
        }
        let block = ((cp >> 8) & 0xFF) as usize;
        if block >= self.blocks.len() {
            return 13.0;
        }
        self.blocks[block].get_width((cp & 0xFF) as u8)
    }
}

impl StringBounder for StringBounderFromWidthTable {
    fn calculate_dimension(&self, font: &UFont, text: &str) -> XDimension2D {
        let size = font.size_2d();
        let factor = size / REFERENCE_SIZE;
        let height = size;
        let mut width = 0.0_f64;
        for cp in text.chars().map(|c| c as u32) {
            width += self.get_char_width(cp);
        }
        // The Java version uses the family for context switching; the width
        // table is sans-serif regardless, so the family does not affect the
        // result here. Reference the context to document intent.
        let _ = font.family(text, UFontContext::Svg);
        XDimension2D::new_or_zero(width * factor, height)
    }

    fn get_file_format(&self) -> FileFormat {
        self.file_format
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_core::u_font::UFont;

    #[test]
    fn empty_text() {
        let sb = StringBounderFromWidthTable::new(FileFormat::SvgDeterministic);
        let font = UFont::sans_serif(16);
        let dim = sb.calculate_dimension(&font, "");
        assert!((dim.width() - 0.0).abs() < 1e-9);
        assert!((dim.height() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn ascii_width() {
        // At 16pt, 'A' (0x41) has width 107 -> 10.7 in the table.
        let sb = StringBounderFromWidthTable::new(FileFormat::SvgDeterministic);
        let font = UFont::sans_serif(16);
        let dim = sb.calculate_dimension(&font, "A");
        assert!((dim.width() - 10.7).abs() < 1e-9, "width was {}", dim.width());
        assert!((dim.height() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn scales_with_font_size() {
        let sb = StringBounderFromWidthTable::new(FileFormat::SvgDeterministic);
        let font16 = UFont::sans_serif(16);
        let font32 = UFont::sans_serif(32);
        let w16 = sb.calculate_dimension(&font16, "Hello").width();
        let w32 = sb.calculate_dimension(&font32, "Hello").width();
        assert!((w32 / w16 - 2.0).abs() < 1e-9, "w16={w16} w32={w32}");
    }

    #[test]
    fn multi_char() {
        // ' ' (0x20) = 0 (zero-width space in this table), 'A' (0x41) = 107 -> 10.7
        let sb = StringBounderFromWidthTable::new(FileFormat::SvgDeterministic);
        let font = UFont::sans_serif(16);
        let dim = sb.calculate_dimension(&font, " A");
        assert!((dim.width() - (0.0 + 10.7)).abs() < 1e-9, "width was {}", dim.width());
    }

    #[test]
    fn high_codepoint_fallback() {
        // Codepoints >= 0xFFFF return 16.0
        let sb = StringBounderFromWidthTable::new(FileFormat::SvgDeterministic);
        let font = UFont::sans_serif(16);
        let dim = sb.calculate_dimension(&font, "\u{10000}");
        assert!((dim.width() - 16.0).abs() < 1e-9);
    }
}
