//! Programmatic API entry point.
//!
//! Ported from `net.sourceforge.plantuml.SourceStringReader`.

use std::io::Write;

use plantuml_core::{FileFormat, FileFormatOption};
use plantuml_preproc::preproc::Defines;

use crate::block_uml::BlockUml;
use crate::block_uml_builder::BlockUmlBuilder;

/// The programmatic entry point for PlantUML.
///
/// Ported from `net.sourceforge.plantuml.SourceStringReader`.
///
/// For Phase 4, only PREPROC format is supported.
pub struct SourceStringReader {
    blocks: Vec<BlockUml>,
}

impl SourceStringReader {
    /// Creates a new `SourceStringReader` from the given source string.
    ///
    /// Ported from `SourceStringReader(String)`.
    pub fn new(source: &str) -> Self {
        Self::with_defines(source, &Defines::new())
    }

    /// Creates a new `SourceStringReader` with the given defines.
    ///
    /// Ported from `SourceStringReader(Defines, String)`.
    pub fn with_defines(source: &str, defines: &Defines) -> Self {
        Self::with_defines_and_dir(source, defines, None)
    }

    /// Creates a new `SourceStringReader` with defines and a current directory.
    pub fn with_defines_and_dir(
        source: &str,
        defines: &Defines,
        current_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let builder = BlockUmlBuilder::with_current_dir(source, defines, current_dir);
        Self {
            blocks: builder.get_block_umls().to_vec(),
        }
    }

    /// Returns the blocks.
    ///
    /// Ported from `SourceStringReader.getBlocks`.
    #[must_use]
    pub fn get_blocks(&self) -> &[BlockUml] {
        &self.blocks
    }

    /// Outputs the diagram image in the given format.
    ///
    /// Ported from `SourceStringReader.outputImage(OutputStream, int, FileFormatOption)`.
    ///
    /// For PREPROC format, writes the preprocessed lines directly.
    /// Returns the diagram description string.
    pub fn output_image(
        &self,
        output: &mut dyn Write,
        num_image: usize,
        file_format_option: &FileFormatOption,
    ) -> Option<String> {
        if self.blocks.is_empty() {
            return None;
        }

        if file_format_option.file_format() == FileFormat::Preproc {
            let first = &self.blocks[0];
            for s in first.get_data() {
                let _ = output.write_all(s.get_string().as_bytes());
                let _ = output.write_all(b"\n");
            }
            return Some("PREPROC".to_string());
        }

        // Other formats not yet supported in Phase 4
        let _ = num_image;
        None
    }
}
