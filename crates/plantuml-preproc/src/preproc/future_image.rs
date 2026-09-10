//! A lazily-decoded image from the stdlib.
//!
//! Ported from `net.sourceforge.plantuml.preproc.FutureImage`.

use std::io::{self, Read};

use crate::stubs::{PortableImage, PortableImageFactory, TYPE_INT_ARGB};

/// A lazily-decoded image: stores raw byte data and decodes on first access.
///
/// Ported from `net.sourceforge.plantuml.preproc.FutureImage`.
pub struct FutureImage {
    width: usize,
    height: usize,
    colors: Vec<i32>,
    data: Option<Vec<u8>>,
    image: Option<PortableImage>,
}

impl FutureImage {
    /// Creates a new `FutureImage` with the given color palette, dimensions, and raw data.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.FutureImage.FutureImage`.
    #[must_use]
    pub fn new(colors: Vec<i32>, width: usize, height: usize, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            colors,
            data: Some(data),
            image: None,
        }
    }

    /// Decodes and returns the image, decoding on first access.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.FutureImage.getNow`.
    pub fn get_now(&mut self) -> PortableImage {
        if self.image.is_none() {
            let image = PortableImageFactory::build(self.width, self.height, TYPE_INT_ARGB);
            if let Some(ref data) = self.data {
                let mut cursor = io::Cursor::new(data);
                for _y in 0..self.height {
                    for _x in 0..self.width {
                        let idx = Self::read2bytes(&mut cursor);
                        // In the full implementation, this would set pixel colors.
                        let _rgb = self.colors.get(idx).copied().unwrap_or(0);
                    }
                }
            }
            self.data = None; // free memory
            self.image = Some(image);
        }
        // Return a clone (PortableImage is a unit struct, so this is free).
        self.image
            .as_ref()
            .map_or(PortableImage, |_| PortableImage)
    }

    fn read2bytes(cursor: &mut io::Cursor<&Vec<u8>>) -> usize {
        let mut buf = [0u8; 1];
        let hi = if cursor.read(&mut buf).is_ok() {
            buf[0] as usize
        } else {
            0
        };
        let lo = if cursor.read(&mut buf).is_ok() {
            buf[0] as usize
        } else {
            0
        };
        (hi << 8) + lo
    }
}
