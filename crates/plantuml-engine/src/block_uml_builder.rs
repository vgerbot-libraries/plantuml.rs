//! Splits preprocessed source into `BlockUml` blocks.
//!
//! Ported from `net.sourceforge.plantuml.BlockUmlBuilder`.

use plantuml_preproc::preproc::Defines;
use plantuml_preproc::string_located::StringLocated;
use plantuml_preproc::stubs::LineLocation;

use crate::block_uml::BlockUml;
use crate::start_utils::StartUtils;

/// Builds `BlockUml` blocks from a source string.
///
/// Ported from `net.sourceforge.plantuml.BlockUmlBuilder`.
///
/// Splits the source into blocks at `@start`/`@end` directives.
/// For Phase 4, the Preprocessor (file inclusion) is not yet wired;
/// lines are passed directly to the `TimLoader`.
pub struct BlockUmlBuilder {
    blocks: Vec<BlockUml>,
}

impl BlockUmlBuilder {
    /// Creates a new `BlockUmlBuilder` from the given source string.
    ///
    /// Ported from `BlockUmlBuilder(List, Charset, Defines, Reader, SFile, String)`.
    pub fn new(source: &str, defines: &Defines) -> Self {
        Self::with_current_dir(source, defines, None)
    }

    /// Creates a new `BlockUmlBuilder` with a current directory for file resolution.
    pub fn with_current_dir(
        source: &str,
        defines: &Defines,
        current_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let lines = Self::create_string_located(source);
        let mut blocks = Vec::new();
        let mut current: Option<Vec<StringLocated>> = None;
        let mut paused = false;

        for s in lines {
            let line_str = s.get_string().to_string();

            if StartUtils::is_start_directive(&line_str) {
                current = Some(Vec::new());
                paused = false;
            }
            if StartUtils::is_pause_directive(&line_str) {
                paused = true;
            }
            if StartUtils::is_exit(&line_str) {
                paused = true;
            }

            if let Some(current_block) = &mut current {
                if paused {
                    // Check for @append directive
                    if let Some(append) = Self::get_possible_append(&line_str) {
                        current_block.push(StringLocated::new(
                            append,
                            LineLocation::new(None, 0),
                        ));
                    }
                } else {
                    current_block.push(s);
                }
            }


            if StartUtils::is_unpause_directive(&line_str) {
                paused = false;
            }

            if StartUtils::is_end_directive(&line_str) {
                if let Some(block_lines) = current.take() {
                    if paused {
                        // When paused, the @end line is included
                        // (already added above if not paused, so add it here if paused)
                    }
                    let block = BlockUml::with_current_dir(block_lines, defines, current_dir.clone());
                    blocks.push(block);
                }
                paused = false;
            }
        }

        Self { blocks }
    }

    /// Returns the built blocks.
    ///
    /// Ported from `BlockUmlBuilder.getBlockUmls`.
    #[must_use]
    pub fn get_block_umls(&self) -> &[BlockUml] {
        &self.blocks
    }

    /// Converts a source string into a list of `StringLocated` lines.
    fn create_string_located(source: &str) -> Vec<StringLocated> {
        let mut result = Vec::new();
        let mut line_num = 0u32;
        for line in source.lines() {
            line_num += 1;
            result.push(StringLocated::new(
                line.to_string(),
                LineLocation::new(Some("string".to_string()), line_num),
            ));
        }
        result
    }

    /// Extracts the content after `@append` or `@a` directive.
    ///
    /// Ported from `StartUtils.getPossibleAppend` (simplified — no regex).
    fn get_possible_append(s: &str) -> Option<String> {
        let trimmed = s.trim_start();
        if let Some(rest) = trimmed.strip_prefix("@append").or_else(|| trimmed.strip_prefix("@a")) {
            // Check word boundary
            if rest.is_empty() || rest.starts_with(char::is_whitespace) {
                return Some(rest.trim().to_string());
            }
        }
        if let Some(rest) = trimmed.strip_prefix("\\append").or_else(|| trimmed.strip_prefix("\\a")) {
            if rest.is_empty() || rest.starts_with(char::is_whitespace) {
                return Some(rest.trim().to_string());
            }
        }
        None
    }
}
