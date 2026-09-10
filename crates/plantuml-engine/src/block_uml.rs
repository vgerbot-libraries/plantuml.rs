//! A single `@start`/`@end` block.
//!
//! Ported from `net.sourceforge.plantuml.BlockUml`.

use plantuml_preproc::preproc::Defines;
use plantuml_preproc::tim::TimLoader;
use plantuml_preproc::StringLocated;

/// One `@start`/`@end` block in the PlantUML source.
///
/// Ported from `net.sourceforge.plantuml.BlockUml`.
///
/// For Phase 4, only PREPROC format is supported. The block runs the
/// TimLoader preprocessor on its raw source and stores the result.
pub struct BlockUml {
    raw_source: Vec<StringLocated>,
    data: Vec<StringLocated>,
    preprocessor_error: bool,
    local_defines: Defines,
}

impl BlockUml {
    /// Creates a new `BlockUml` from the given source lines.
    ///
    /// Ported from `BlockUml(DefinitionsContainer, PathSystem, List<StringLocated>, Defines, Previous, Charset)`.
    ///
    /// Runs the TimLoader preprocessor on the source lines.
    pub fn new(strings: Vec<StringLocated>, defines: &Defines) -> Self {
        Self::with_current_dir(strings, defines, None)
    }

    /// Creates a new `BlockUml` with a current directory for file resolution.
    pub fn with_current_dir(
        strings: Vec<StringLocated>,
        defines: &Defines,
        current_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let raw_source = strings;

        let mut loader = TimLoader::new(defines);
        if let Some(dir) = current_dir {
            loader.set_current_dir(dir);
        }
        loader.load(raw_source.clone());

        let data = loader.get_result_list().cloned().unwrap_or_default();
        let preprocessor_error = loader.is_preprocessor_error();

        Self {
            raw_source,
            data,
            preprocessor_error,
            local_defines: defines.clone(),
        }
    }

    /// Returns the preprocessed data lines.
    ///
    /// Ported from `BlockUml.getData`.
    #[must_use]
    pub fn get_data(&self) -> &[StringLocated] {
        &self.data
    }

    /// Returns `true` if a preprocessor error occurred.
    #[must_use]
    pub fn is_preprocessor_error(&self) -> bool {
        self.preprocessor_error
    }

    /// Returns the raw source lines (before preprocessing).
    #[must_use]
    pub fn get_raw_source(&self) -> &[StringLocated] {
        &self.raw_source
    }

    /// Returns the local defines.
    #[must_use]
    pub fn get_local_defines(&self) -> &Defines {
        &self.local_defines
    }
}

impl Clone for BlockUml {
    fn clone(&self) -> Self {
        Self {
            raw_source: self.raw_source.clone(),
            data: self.data.clone(),
            preprocessor_error: self.preprocessor_error,
            local_defines: self.local_defines.clone(),
        }
    }
}
