//! SPM (Stdlib Package Manager) channel — resource stream channels.
//!
//! Ported from `net.sourceforge.plantuml.preproc.spm.SpmChannel`.

use std::io::{self, Read};

/// Channels for different resource types in the stdlib.
///
/// Ported from `net.sourceforge.plantuml.preproc.spm.SpmChannel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpmChannel {
    Info,
    Puml,
    Json,
    Sprite,
    Svg,
    Image,
}

impl SpmChannel {
    /// Returns the file name for this channel (e.g. `info.spm`).
    fn get_file_name(&self) -> String {
        let name = match self {
            Self::Info => "info",
            Self::Puml => "puml",
            Self::Json => "json",
            Self::Sprite => "sprite",
            Self::Svg => "svg",
            Self::Image => "image",
        };
        format!("{}.spm", name)
    }

    /// Returns an input stream for the given library name and channel.
    ///
    /// In Java, this reads from a Brotli-compressed resource file.
    /// In Rust, we read from the file system or embedded resources.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.spm.SpmChannel.getInternalInputStream`.
    pub fn get_internal_stream(&self, libname: &str) -> io::Result<Box<dyn Read>> {
        let path = format!("stdlib/{}/{}", libname, self.get_file_name());
        Self::input_stream(&path)
    }

    /// Opens an input stream for a resource path.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.spm.SpmChannel.inputStream`.
    pub fn input_stream(path: &str) -> io::Result<Box<dyn Read>> {
        // Try to read from the file system first.
        // In the full implementation, this would also try embedded resources.
        let file = std::fs::File::open(path)?;
        Ok(Box::new(io::BufReader::new(file)))
    }
}
