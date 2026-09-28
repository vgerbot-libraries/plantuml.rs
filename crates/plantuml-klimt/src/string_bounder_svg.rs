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

/// Per-character widths (in pixels at 16 pt) for ASCII 0x00–0x7F in ITALIC style,
/// measured from Java AWT `FontMetrics.getStringBounds()` with fractional metrics.
#[allow(clippy::unreadable_literal)]
const AWT_ASCII_ITALIC_WIDTHS: [f64; 128] = [
    // 0x00–0x0F
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    // 0x10–0x1F
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    // 0x20–0x2F
    4.1600, 4.1760, 6.2720, 10.3360, 8.8160, 12.7840, 10.7680, 3.5200,
    4.6400, 4.6400, 8.8160, 9.1520, 4.0960, 5.0080, 4.0960, 5.6960,
    // 0x30–0x3F
    8.8160, 8.8160, 8.8160, 8.8160, 8.8160, 8.8160, 8.8160, 8.8160,
    8.8160, 8.8160, 4.0960, 4.0960, 9.1520, 9.1520, 9.1520, 6.8800,
    // 0x40–0x4F
    13.5680, 8.9920, 9.6000, 9.3920, 10.6720, 8.2240, 7.6320, 10.8480,
    10.8800, 5.1840, 4.3680, 8.9920, 7.6480, 13.4560, 11.3120, 11.5360,
    // 0x50–0x5F
    9.0720, 11.5360, 9.1680, 8.0800, 8.0160, 10.8320, 8.8320, 13.6960,
    8.4320, 8.1440, 8.4960, 4.6400, 5.6960, 4.6400, 9.1520, 6.3200,
    // 0x60–0x6F
    4.4480, 9.0880, 9.2640, 7.2480, 9.2640, 7.9840, 5.0880, 9.2640,
    9.2640, 4.1280, 4.1280, 7.9200, 4.1280, 14.0000, 9.2640, 9.0080,
    // 0x70–0x7F
    9.2640, 9.2640, 6.3680, 6.9120, 5.3120, 9.2640, 7.4720, 11.5680,
    7.7280, 7.4720, 7.1200, 5.6000, 8.8160, 5.6000, 9.1520, 0.0,
];

/// Per-character widths (pixels, font size 11, chars 0x20–0x7E) measured from
/// Java AWT `getStringBounds` with the jar's exact `FontRenderContext`.
///
/// AWT quantizes glyph advances at each ppem, so scaled 16 pt widths are
/// ~0.0004 px short; a size-exact table is required for byte-parity.
#[allow(clippy::unreadable_literal)]
const AWT_SIZE_11_PLAIN: [f64; 95] = [
    2.859985, 2.958984, 4.487961, 7.105942, 6.291946, 9.140930, 8.051941, 2.474976,
    3.299973, 3.299973, 6.060959, 6.291946, 2.947983, 3.541977, 2.947983, 4.091965,
    6.291946, 6.291946, 6.291946, 6.291946, 6.291946, 6.291946, 6.291946, 6.291946,
    6.291946, 6.291946, 2.947983, 2.947983, 6.291946, 6.291946, 6.291946, 4.773972,
    9.888931, 7.028946, 7.149948, 6.951950, 8.029938, 6.115952, 5.708954, 8.007935,
    8.150940, 3.728973, 3.002975, 6.808945, 5.763962, 9.976929, 8.359940, 8.590942,
    6.654953, 8.590942, 6.841949, 6.038956, 6.115952, 8.040939, 6.599945, 10.229919,
    6.445953, 6.225952, 6.291946, 3.618973, 4.091965, 3.618973, 6.291946, 4.883957,
    3.090973, 6.170959, 6.764954, 5.279968, 6.764954, 6.203949, 3.783966, 6.764954,
    6.797943, 2.837982, 2.837982, 5.873962, 2.837982, 10.284927, 6.797943, 6.654953,
    6.764954, 6.764954, 4.542969, 5.268967, 3.970978, 6.797943, 5.587952, 8.645935,
    5.818954, 5.609955, 5.169968, 4.179962, 6.060959, 4.179962, 6.291946,
];

/// Per-character widths (pixels, font size 12, chars 0x20–0x7E).
#[allow(clippy::unreadable_literal)]
const AWT_SIZE_12_PLAIN: [f64; 95] = [
    3.120026, 3.228027, 4.896042, 7.752060, 6.864044, 9.972076, 8.784058, 2.700012,
    3.600021, 3.600021, 6.612045, 6.864044, 3.216019, 3.864029, 3.216019, 4.464035,
    6.864044, 6.864044, 6.864044, 6.864044, 6.864044, 6.864044, 6.864044, 6.864044,
    6.864044, 6.864044, 3.216019, 3.216019, 6.864044, 6.864044, 6.864044, 5.208038,
    10.788071, 7.668060, 7.800049, 7.584061, 8.760056, 6.672043, 6.228043, 8.736069,
    8.892059, 4.068024, 3.276016, 7.428055, 6.288040, 10.884079, 9.120071, 9.372070,
    7.260056, 9.372070, 7.464050, 6.588043, 6.672043, 8.772064, 7.200058, 11.160080,
    7.032043, 6.792053, 6.864044, 3.948029, 4.464035, 3.948029, 6.864044, 5.328033,
    3.372025, 6.732040, 7.380051, 5.760040, 7.380051, 6.768051, 4.128036, 7.380051,
    7.416046, 3.096024, 3.096024, 6.408051, 3.096024, 11.220078, 7.416046, 7.260056,
    7.380051, 7.380051, 4.956039, 5.748047, 4.332031, 7.416046, 6.096039, 9.432068,
    6.348038, 6.120041, 5.640045, 4.560028, 6.612045, 4.560028, 6.864044,
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

    fn get_char_width(&self, cp: u32, italic: bool) -> f64 {
        if cp < 128 {
            return if italic {
                AWT_ASCII_ITALIC_WIDTHS[cp as usize]
            } else {
                AWT_ASCII_WIDTHS[cp as usize]
            };
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

    /// Sums the size-exact ASCII width for `text`, or `None` if the size has
    /// no exact table or the text contains a character outside 0x20–0x7E.
    fn exact_ascii_width(size: f64, text: &str) -> Option<f64> {
        let table: &[f64; 95] = if (size - 11.0).abs() < f64::EPSILON {
            &AWT_SIZE_11_PLAIN
        } else if (size - 12.0).abs() < f64::EPSILON {
            &AWT_SIZE_12_PLAIN
        } else {
            return None;
        };
        let mut width = 0.0_f64;
        for cp in text.chars().map(|c| c as u32) {
            if !(0x20..=0x7e).contains(&cp) {
                return None;
            }
            width += table[(cp - 0x20) as usize];
        }
        Some(width)
    }
}

impl StringBounder for StringBounderSvg {
    fn calculate_dimension(&self, font: &UFont, text: &str) -> XDimension2D {
        let size = font.size_2d();
        // Java AWT returns a visual height ≈ size * 1.362 (ascent + descent).
        let height = size * 1.362;
        let italic = font.style().italic;

        // Size-exact tables reproduce AWT's per-ppem advance quantization.
        if !italic {
            if let Some(width) = Self::exact_ascii_width(size, text) {
                let _ = font.family(text, plantuml_core::u_font::UFontContext::Svg);
                return XDimension2D::new_or_zero(width, height);
            }
        }

        let factor = size / REFERENCE_SIZE;
        let mut width = 0.0_f64;
        for cp in text.chars().map(|c| c as u32) {
            width += self.get_char_width(cp, italic);
        }
        // Scaled 16 pt per-character advances slightly over-state the whole
        // string; a measured correction factor (plain vs italic, 14 pt).
        let correction = if italic { 0.000_003_76 } else { 0.000_006_3 };
        let width = width * (1.0 - correction) * factor;
        let _ = font.family(text, plantuml_core::u_font::UFontContext::Svg);
        XDimension2D::new_or_zero(width, height)
    }

    fn get_file_format(&self) -> FileFormat {
        self.file_format
    }
}
