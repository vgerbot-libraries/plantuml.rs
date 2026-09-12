//! SVG string bounder using Java AWT-derived font metrics.
//!
//! Matches the text widths produced by Java's `StringBounderSvg`, which uses
//! `FontMetrics.getStringBounds()` with `FRACTIONALMETRICS_ON`. The per-character
//! widths below were measured from Java AWT SansSerif at 16 pt reference size
//! and are scaled by `font_size / 16.0` at measurement time.
//!
//! For non-ASCII code points, falls back to the deterministic width table.

use plantuml_core::file_format::FileFormat;
use plantuml_core::geom::XDimension2D;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;

use crate::unicode_font_width_sans_serif::SANS_SERIF;
use crate::unicode_block::UnicodeBlock;

/// Reference font size for the width table (16 pt).
const REFERENCE_SIZE: f64 = 16.0;

/// Per-character widths (in pixels at 16 pt) for ASCII 0x00–0x7F, measured
/// from Java AWT `FontMetrics.getStringBounds()` with fractional metrics.
///
/// Characters 0x00–0x1F and 0x7F–0xFF are zero (non-printable / non-ASCII).
/// For code points ≥ 0x80, the bounder falls back to `StringBounderFromWidthTable`.
#[allow(clippy::unreadable_literal)]
const AWT_ASCII_WIDTHS: [f64; 128] = [
    // 0x00–0x0F
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    // 0x10–0x1F
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    // 0x20–0x2F
    4.1600036621, 4.3040161133, 6.5280151367, 10.3360137939, 9.1520233154, 13.2960205078, 11.7120208740, 3.6000061035,
    4.8000030518, 4.8000030518, 8.8160247803, 9.1520233154, 4.2880096436, 5.1520080566, 4.2880096436, 5.9520111084,
    // 0x30–0x3F
    9.1520233154, 9.1520233154, 9.1520233154, 9.1520233154, 9.1520233154, 9.1520233154, 9.1520233154, 9.1520233154,
    9.1520233154, 9.1520233154, 4.2880096436, 4.2880096436, 9.1520233154, 9.1520233154, 9.1520233154, 6.9440155029,
    // 0x40–0x4F
    14.3840332031, 10.2240142822, 10.4000244141, 10.1120147705, 11.6800231934, 8.8960113525, 8.3040161133, 11.6480255127,
    11.8560180664, 5.4240112305, 4.3680114746, 9.9040222168, 8.3840179443, 14.5120239258, 12.1600189209, 12.4960327148,
    // 0x50–0x5F
    9.6800231934, 12.4960327148, 9.9520263672, 8.7840118408, 8.8960113525, 11.6960296631, 9.6000213623, 14.8800354004,
    9.3760223389, 9.0560150146, 9.1520233154, 5.2640075684, 5.9520111084, 5.2640075684, 9.1520233154, 7.1040191650,
    // 0x60–0x6F
    4.4960021973, 8.9760131836, 9.8400268555, 7.6800231934, 9.8400268555, 9.0240173340, 5.5040130615, 9.8400268555,
    9.8880157471, 4.1280059814, 4.1280059814, 8.5440216064, 4.1280059814, 14.9600372314, 9.8880157471, 9.6800231934,
    // 0x70–0x7F
    9.8400268555, 9.8400268555, 6.6080169678, 7.6640167236, 5.7760162354, 9.8880157471, 8.1280212402, 12.5760192871,
    8.4640197754, 8.1600189209, 7.5200195313, 6.0800170898, 8.8160247803, 6.0800170898, 9.1520233154, 0.0,
];

/// A `StringBounder` that matches Java AWT font metrics for SVG output.
///
/// Uses pre-computed AWT character widths for ASCII text and falls back to
/// the deterministic width table for non-ASCII code points.
pub struct StringBounderSvg {
    file_format: FileFormat,
    /// Pre-decoded Unicode blocks for non-ASCII fallback.
    fallback_blocks: Vec<UnicodeBlock>,
}

impl StringBounderSvg {
    #[must_use]
    pub fn new(file_format: FileFormat) -> Self {
        let fallback_blocks = SANS_SERIF
            .iter()
            .map(|raw| UnicodeBlock::new(raw))
            .collect();
        Self {
            file_format,
            fallback_blocks,
        }
    }

    fn get_char_width(&self, cp: u32) -> f64 {
        if cp < 128 {
            return AWT_ASCII_WIDTHS[cp as usize];
        }
        if cp >= 0xFFFF {
            return 16.0;
        }
        let block = ((cp >> 8) & 0xFF) as usize;
        if block >= self.fallback_blocks.len() {
            return 13.0;
        }
        self.fallback_blocks[block].get_width((cp & 0xFF) as u8)
    }
}

impl StringBounder for StringBounderSvg {
    fn calculate_dimension(&self, font: &UFont, text: &str) -> XDimension2D {
        let size = font.size_2d();
        let factor = size / REFERENCE_SIZE;
        let height = size;
        let mut width = 0.0_f64;
        for cp in text.chars().map(|c| c as u32) {
            width += self.get_char_width(cp);
        }
        let width = width * factor;
        let _ = font.family(text, plantuml_core::u_font::UFontContext::Svg);
        XDimension2D::new_or_zero(width, height)
    }

    fn get_file_format(&self) -> FileFormat {
        self.file_format
    }
}
