//! RLE-decoded Unicode width block.
//!
//! Ported from: net/sourceforge/plantuml/klimt/drawing/font/UnicodeBlock.java

/// A decoded width table for one 256-code-point Unicode block.
///
/// Blocks with a single element are uniform-width. Blocks with fewer than 256
/// elements use run-length encoding: pairs of `(count, value)` bytes that
/// expand to `count` copies of `value`.
pub struct UnicodeBlock {
    data: Vec<u8>,
}

impl UnicodeBlock {
    /// Decode a raw block slice into a 256-entry width table.
    #[must_use]
    pub fn new(raw: &[u8]) -> Self {
        let data = if raw.len() == 1 || raw.len() >= 256 {
            // Uniform block, or already fully expanded.
            if raw.len() == 1 {
                vec![raw[0]; 256]
            } else {
                raw.to_vec()
            }
        } else {
            decode_rle(raw)
        };
        Self { data }
    }

    /// Width of `ch` (low byte of the code point) in tenths of a pixel.
    #[must_use]
    pub fn get_width(&self, ch: u8) -> f64 {
        let width = if self.data.len() == 1 {
            u16::from(self.data[0])
        } else {
            u16::from(self.data[usize::from(ch)])
        };
        f64::from(width) / 10.0
    }

    /// Raw width of `ch` (low byte of the code point) in tenths of a pixel.
    #[must_use]
    pub fn get_width_raw(&self, ch: u8) -> u16 {
        if self.data.len() == 1 {
            u16::from(self.data[0])
        } else {
            u16::from(self.data[usize::from(ch)])
        }
    }
}

fn decode_rle(data: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; 256];
    let mut idx = 0usize;
    let mut i = 0usize;
    while i < data.len() {
        let count = usize::from(data[i]);
        let value = data[i + 1];
        for _ in 0..count {
            if idx < 256 {
                result[idx] = value;
                idx += 1;
            }
        }
        i += 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_block() {
        let b = UnicodeBlock::new(&[81]);
        assert!((b.get_width(0) - 8.1).abs() < 1e-9);
        assert!((b.get_width(255) - 8.1).abs() < 1e-9);
    }

    #[test]
    fn full_block() {
        let raw: Vec<u8> = (0..=255).collect();
        let b = UnicodeBlock::new(&raw);
        assert!((b.get_width(0) - 0.0).abs() < 1e-9);
        assert!((b.get_width(100) - 10.0).abs() < 1e-9);
        assert!((b.get_width(255) - 25.5).abs() < 1e-9);
    }

    #[test]
    fn rle_block() {
        // 3 copies of 50, then 2 copies of 60
        let b = UnicodeBlock::new(&[3, 50, 2, 60]);
        assert!((b.get_width(0) - 5.0).abs() < 1e-9);
        assert!((b.get_width(2) - 5.0).abs() < 1e-9);
        assert!((b.get_width(3) - 6.0).abs() < 1e-9);
        assert!((b.get_width(4) - 6.0).abs() < 1e-9);
    }
}
