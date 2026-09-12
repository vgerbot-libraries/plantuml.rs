//! Test-only helper that invokes Java PlantUML to render SVG from source text.
//!
//! Used by parity tests to generate the Java reference SVG, which is then
//! compared (after normalization) against the Rust-rendered SVG.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Resolves the PlantUML jar path.
///
/// Checks the `PLANTUML_JAR` environment variable first, then falls back to
/// `tests/vendor/plantuml.jar` relative to the crate manifest directory.
fn jar_path() -> PathBuf {
    if let Ok(p) = std::env::var("PLANTUML_JAR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("vendor")
        .join("plantuml.jar")
}

/// Returns `true` if Java is on PATH **and** the PlantUML jar is available
/// (either via `PLANTUML_JAR` or the project-local copy).
///
/// Tests call this to skip gracefully when Java is not installed (e.g. CI
#[allow(dead_code)]
pub fn is_available() -> bool {
    let jar = jar_path();
    if !jar.exists() {
        return false;
    }
    Command::new("java")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Renders `source` (PlantUML text) to an SVG string via Java PlantUML.
///
/// Spawns `java -jar <jar> -tsvg -pipe` with `source` piped to stdin.
///
/// Returns `Some(svg_string)` on success, or `None` if Java/jar is unavailable
/// or the command fails.
pub fn render_svg(source: &str) -> Option<String> {
    let jar = jar_path();
    if !jar.exists() {
        return None;
    }

    let mut child = Command::new("java")
        .arg("-jar")
        .arg(&jar)
        .arg("-tsvg")
        .arg("-pipe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    // Write source to stdin.
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(source.as_bytes());
        // stdin is dropped here, closing the pipe.
    }

    let output = child.wait_with_output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}
