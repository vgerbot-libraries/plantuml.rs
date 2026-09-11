//! Stub types for external packages not yet ported.
//!
//! These types are minimal placeholders that allow the tim/ core to compile.
//! They will be replaced by real implementations from other crates/modules
//! as the migration progresses.

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.utils.LineLocation
// ---------------------------------------------------------------------------

/// Minimal line location info.
///
/// Ported from `net.sourceforge.plantuml.utils.LineLocation`.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LineLocation {
    pub file: Option<String>,
    pub line: u32,
}

impl LineLocation {
    #[must_use]
    pub fn new(file: Option<String>, line: u32) -> Self {
        Self { file, line }
    }
}


// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.nio.PathSystem
// ---------------------------------------------------------------------------

/// Stub for `PathSystem` — file resolution and include tracking.
///
/// Ported from `net.sourceforge.plantuml.nio.PathSystem`.
/// Stub for `PathSystem` — manages file path resolution for !include directives.
///
/// Ported from `net.sourceforge.plantuml.security.PathSystem`.
#[derive(Debug, Clone, Default)]
pub struct PathSystem {
    /// Current directory for resolving relative file paths.
    pub current_dir: Option<std::path::PathBuf>,
}

impl PathSystem {
    #[must_use]
    pub fn new() -> Self {
        Self { current_dir: None }
    }

    /// Sets the current directory.
    pub fn set_current_dir(&mut self, dir: impl Into<std::path::PathBuf>) {
        self.current_dir = Some(dir.into());
    }

    /// Resolves a file path relative to the current directory.
    /// Returns the resolved path if it exists.
    pub fn resolve_file(&self, filename: &str) -> Option<std::path::PathBuf> {
        let path = std::path::Path::new(filename);
        if path.is_absolute() {
            return path.exists().then(|| path.to_path_buf());
        }
        if let Some(dir) = &self.current_dir {
            let resolved = dir.join(filename);
            if resolved.exists() {
                return Some(resolved);
            }
        }
        path.exists().then(|| path.to_path_buf())
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.DefinitionsContainer
// ---------------------------------------------------------------------------

/// Stub for `DefinitionsContainer` — manages `@startdef` definitions.
///
/// Ported from `net.sourceforge.plantuml.DefinitionsContainer`.
#[derive(Debug, Clone, Default)]
pub struct DefinitionsContainer;

impl DefinitionsContainer {
    /// Returns the definition lines for the given name, if it exists.
    pub fn get_definition(&self, _name: &str) -> Option<Vec<String>> {
        None
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.skin.Pragma
// ---------------------------------------------------------------------------

/// Stub for Pragma — pragma directives.
///
/// Ported from `net.sourceforge.plantuml.skin.Pragma`.
#[derive(Debug, Clone, Default)]
pub struct Pragma;

impl Pragma {
    /// Returns `true` if the `legacy_replace_backslash_n_by_newline` pragma is active.
    #[must_use]
    pub fn legacy_replace_backslash_n_by_newline() -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.jaws.Jaws
// ---------------------------------------------------------------------------

/// Jaws constants used by the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.jaws.Jaws`.
pub struct Jaws;

impl Jaws {
    /// The block E1 newline character used internally.
    pub const BLOCK_E1_NEWLINE: char = '\u{E001}';
    /// The real backslash replacement character.
    pub const BLOCK_E1_REAL_BACKSLASH: char = '\u{E002}';
    /// Trace flag.
    pub const TRACE: bool = false;
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.command.CommandExecutionResult
// ---------------------------------------------------------------------------

/// Stub for `CommandExecutionResult`.
///
/// Ported from `net.sourceforge.plantuml.command.CommandExecutionResult`.
#[derive(Debug)]
pub struct CommandExecutionResult;

impl CommandExecutionResult {
    pub fn error(_msg: &str) -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.teavm.TeaVM
// ---------------------------------------------------------------------------

/// Stub for `TeaVM` platform checks.
///
/// Ported from `net.sourceforge.plantuml.teavm.TeaVM`.
pub struct TeaVM;

impl TeaVM {
    /// Returns `true` if running under `TeaVM` (browser/WASM).
    #[must_use]
    pub fn is_teavm() -> bool {
        false
    }

    /// Assertion helper — always returns `true` in native mode.
    #[must_use]
    pub fn a() -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.log.Logme
// ---------------------------------------------------------------------------

/// Stub for Logme — error logging.
///
/// Ported from `net.sourceforge.plantuml.log.Logme`.
pub struct Logme;

impl Logme {
    pub fn error<E: std::fmt::Debug>(_e: E) {}
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.utils.Log
// ---------------------------------------------------------------------------

/// Stub for Log — user logging.
///
/// Ported from `net.sourceforge.plantuml.utils.Log`.
pub struct Log;

impl Log {
    pub fn user_log<F: FnOnce() -> String>(_f: F) {}
    pub fn info<F: FnOnce() -> String>(_f: F) {}
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.warning.Warning
// ---------------------------------------------------------------------------

/// Stub for Warning.
///
/// Ported from `net.sourceforge.plantuml.warning.Warning`.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Warning(pub String);

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.FoxSignature
// ---------------------------------------------------------------------------

/// Stub for `FoxSignature` — fast-reject signature.
///
/// Ported from `net.sourceforge.plantuml.FoxSignature`.
pub struct FoxSignature;

impl FoxSignature {
    #[must_use]
    pub fn get_fox_signature_from_real_string(_s: &str) -> u64 {
        0
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.StringUtils
// ---------------------------------------------------------------------------

/// String utility functions.
///
/// Ported from `net.sourceforge.plantuml.StringUtils`.
pub struct StringUtils;

impl StringUtils {
    /// Returns `true` if the string ends with a backslash.
    #[must_use]
    pub fn ends_with_backslash(s: &str) -> bool {
        s.ends_with('\\')
    }

    /// Trims leading and trailing whitespace from a string.
    #[must_use]
    pub fn trin(s: &str) -> String {
        s.trim().to_string()
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.theme.Theme
// ---------------------------------------------------------------------------

/// Stub for Theme — theme loading.
///
/// Ported from `net.sourceforge.plantuml.theme.Theme`.
#[derive(Debug, Default)]
pub struct Theme {
    pub metadata: serde_json::Value,
}

impl Theme {
    pub fn read_line(&mut self) -> Option<crate::StringLocated> {
        None
    }

    pub fn get_metadata(&self) -> &serde_json::Value {
        &self.metadata
    }

    pub fn close(&mut self) {}
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.theme.ThemeUtils
// ---------------------------------------------------------------------------

/// Stub for `ThemeUtils`.
///
/// Ported from `net.sourceforge.plantuml.theme.ThemeUtils`.
pub struct ThemeUtils;

impl ThemeUtils {
    pub fn load_theme(
        _path_system: &PathSystem,
        _name: &str,
        _from: Option<&str>,
        _location: &crate::StringLocated,
    ) -> Option<Theme> {
        None
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.security.SFile / SURL
// ---------------------------------------------------------------------------

/// Stub for `SFile`.
#[derive(Debug, Clone)]
pub struct SFile {
    pub path: String,
}

impl SFile {
    pub fn exists(&self) -> bool {
        false
    }
    pub fn is_directory(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.nio.InputFile
// ---------------------------------------------------------------------------

/// Stub for `InputFile`.
#[derive(Debug, Clone)]
pub struct InputFile {
    pub path: String,
}

impl InputFile {
    pub fn get_parent_folder(&self) -> String {
        String::new()
    }
    pub fn get_reader(&self, _charset: &str) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.FileSystem
// ---------------------------------------------------------------------------

/// Stub for `FileSystem`.
pub struct FileSystem;

impl FileSystem {
    pub fn get_file(_path: &str) -> SFile {
        SFile {
            path: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Empty set helper
// ---------------------------------------------------------------------------

/// Returns an empty `HashSet<String>`.
#[must_use]
pub fn empty_string_set() -> HashSet<String> {
    HashSet::new()
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.utils.StartUtils
// ---------------------------------------------------------------------------

/// Start/end directive utilities.
///
/// Ported from `net.sourceforge.plantuml.utils.StartUtils`.
pub struct StartUtils;

impl StartUtils {
    /// The pause pattern regex string.
    pub const PAUSE_PATTERN: &'static str = r"((?:\W|\<[^<>]*\>)*)[@\\]unpause";

    /// Returns `true` if the line is a `@start` directive.
    #[must_use]
    pub fn is_start_directive(s: &str) -> bool {
        let n = s.len();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < n && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= n {
            return false;
        }
 let c = bytes[i];
        if c != b'@' && c != b'\\' {
            return false;
        }
        // need '@' + "start" + at least one char after
        i + 6 < n && s[i + 1..].starts_with("start")
    }

    /// Returns `true` if the line is an `@end` directive.
    #[must_use]
    pub fn is_end_directive(s: &str) -> bool {
        Self::starts_with_directive_keyword(s, "end")
    }

    /// Returns the text before `@start` if present, otherwise `None`.
    #[must_use]
    pub fn before_start_uml(s: &str) -> Option<String> {
        let n = s.len();
        let mut inside = false;
        let chars: Vec<char> = s.chars().collect();
        for i in 0..n {
            if Self::starts_with_directive_keyword(&s[i..], "start") {
                return Some(s[..i].to_string());
            }
            let c = chars[i];
            if inside {
                if c == '>' {
                    inside = false;
                }
                continue;
            }
            if c == '<' {
                inside = true;
            } else if Self::is_word_or_tilde(c) {
                return None;
            }
        }
        None
    }

    fn is_word_or_tilde(c: char) -> bool {
        c == '~' || c.is_alphanumeric() || c == '_'
    }

    fn starts_with_directive_keyword(text: &str, keyword: &str) -> bool {
        let n = text.len();
        let mut i = 0;
        let bytes = text.as_bytes();
        while i < n {
            let c = bytes[i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }
            if c != b'@' && c != b'\\' {
                return false;
            }
            let start = i + 1;
            if start + keyword.len() > n {
                return false;
            }
            return text[start..start + keyword.len()] == *keyword;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.text.BackSlash
// ---------------------------------------------------------------------------

/// Backslash translation utilities.
///
/// Ported from `net.sourceforge.plantuml.text.BackSlash`.
pub struct BackSlash;

impl BackSlash {
    /// The private-use block base used to hide backslash sequences.
    pub const PRIVATE_BLOCK: char = '\u{E000}';

    /// Translates `\n` sequences by escaping the `n` into the private block.
    pub fn translate_back_slashes(s: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut i = 0;
        while i < n {
            let c = chars[i];
            if c == '\\' && i + 1 < n && chars[i + 1] == 'n' {
                result.push('\\');
                result.push(Self::translate_char(chars[i + 1]));
                i += 2;
            } else {
                result.push(c);
                i += 1;
            }
        }
        result
    }

    /// Reverses the private-block escaping.
    pub fn untranslate_back_slashes(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            if c > Self::PRIVATE_BLOCK && c < '\u{E07F}' {
                result.push(char::from_u32(c as u32 - Self::PRIVATE_BLOCK as u32).unwrap_or(c));
            } else {
                result.push(c);
            }
        }
        result
    }

    fn translate_char(c: char) -> char {
        char::from_u32(Self::PRIVATE_BLOCK as u32 + c as u32).unwrap_or(c)
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.warning.WarningHandler
// ---------------------------------------------------------------------------

/// Trait for handling warnings during preprocessing.
///
/// Ported from `net.sourceforge.plantuml.warning.WarningHandler`.
pub trait WarningHandler {
    fn add_warning(&mut self, warning: Warning);
    fn get_warnings(&self) -> Vec<Warning>;
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.FileUtils
// ---------------------------------------------------------------------------

/// File I/O utilities.
///
/// Ported from `net.sourceforge.plantuml.FileUtils`.
pub struct FileUtils;

impl FileUtils {
    /// Reads all lines from a byte slice (UTF-8).
    pub fn read_strings(data: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(data)
            .lines()
            .map(String::from)
            .collect()
    }

    /// Reads exactly `len` bytes from a slice, returning the sub-slice as a Vec.
    pub fn read_exactly(data: &[u8], len: usize) -> Vec<u8> {
        data.get(..len).map_or(Vec::new(), <[u8]>::to_vec)
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.security.SURL
// ---------------------------------------------------------------------------

/// Stub for SURL — secure URL.
///
/// Ported from `net.sourceforge.plantuml.security.SURL`.
#[derive(Debug, Clone)]
pub struct SURL {
    pub url: String,
}

impl SURL {
    pub fn open_stream(&self) -> Option<Vec<u8>> {
        None
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.url.clone()
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.klimt.sprite.Sprite
// ---------------------------------------------------------------------------

/// Stub for Sprite — a monochrome or color sprite image.
///
/// Ported from `net.sourceforge.plantuml.klimt.sprite.Sprite`.
pub trait Sprite: Send + Sync {
    fn as_text_block(&self) -> String;
}

/// Stub for `SpriteMonochrome`.
///
/// Ported from `net.sourceforge.plantuml.klimt.sprite.SpriteMonochrome`.
pub struct SpriteMonochrome {
    pub width: usize,
    pub height: usize,
    pub gray: Vec<Vec<u8>>,
}

impl SpriteMonochrome {
    pub fn new(width: usize, height: usize, _levels: usize) -> Self {
        Self {
            width,
            height,
            gray: vec![vec![0; height]; width],
        }
    }

    pub fn set_gray(&mut self, x: usize, y: usize, val: u8) {
        if x < self.width && y < self.height {
            self.gray[x][y] = val;
        }
    }
}

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.klimt.awt.PortableImage
// ---------------------------------------------------------------------------

/// Stub for `PortableImage`.
///
/// Ported from `net.sourceforge.plantuml.klimt.awt.PortableImage`.
pub struct PortableImage;

/// Stub for `PortableImageFactory`.
///
/// Ported from `net.sourceforge.plantuml.klimt.awt.PortableImageFactory`.
pub struct PortableImageFactory;

impl PortableImageFactory {
    pub fn build(_width: usize, _height: usize, _type: usize) -> PortableImage {
        PortableImage
    }
}

/// Image type constant.
pub const TYPE_INT_ARGB: usize = 0;

// ---------------------------------------------------------------------------
// net.sourceforge.plantuml.version.Version
// ---------------------------------------------------------------------------

/// Returns the `PlantUML` version string.
pub fn version_string() -> String {
    "1.2024.7".to_string()
}
