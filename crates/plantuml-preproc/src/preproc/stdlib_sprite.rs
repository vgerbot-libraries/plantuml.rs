//! A sprite loaded from the stdlib.
//!
//! Ported from `net.sourceforge.plantuml.preproc.StdlibSprite`.

use crate::stubs::{Sprite, SpriteMonochrome};

/// A monochrome sprite loaded from the stdlib's SPM data.
///
/// Ported from `net.sourceforge.plantuml.preproc.StdlibSprite`.
pub struct StdlibSprite {
    sprite: Option<SpriteMonochrome>,
    data: Option<Vec<u8>>,
    width: usize,
    height: usize,
}

impl StdlibSprite {
    /// Creates a new `StdlibSprite` with the given dimensions and raw data.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.StdlibSprite.StdlibSprite`.
    #[must_use]
    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Self {
        Self {
            sprite: None,
            data: Some(data),
            width,
            height,
        }
    }

    /// Decodes the sprite on first access.
    pub fn decode(&mut self) {
        if self.sprite.is_none() {
            let mut sprite = SpriteMonochrome::new(self.width, self.height, 16);
            if let Some(ref data) = self.data {
                let nb_lines = self.height.div_ceil(2);
                let mut pos = 0;
                for j in 0..nb_lines {
                    for i in 0..self.width {
                        if pos < data.len() {
                            let b = data[pos];
                            let b1 = (b & 0xF0) >> 4;
                            let b2 = b & 0x0F;
                            sprite.set_gray(i, j * 2, b1);
                            sprite.set_gray(i, j * 2 + 1, b2);
                            pos += 1;
                        }
                    }
                }
            }
            self.data = None;
            self.sprite = Some(sprite);
        }
    }
}

impl Sprite for StdlibSprite {
    fn as_text_block(&self) -> String {
        // In the full implementation, this delegates to the decoded SpriteMonochrome.
        String::new()
    }
}
