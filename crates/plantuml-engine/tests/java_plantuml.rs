//! Test-only helper that invokes Java PlantUML to render output from source text.
//!
//! Used by parity tests to generate the Java reference output, which is then
//! compared (after normalization) against the Rust-rendered output.
//!
//! Ported from: test/vega/VegaTest.java (JAR invocation logic)

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
/// without Java).
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

/// The CLI flag for each output format.
///
/// Maps a vega YAML `output` value to the corresponding `-t<format>` CLI flag
/// used by `java -jar plantuml.jar -t<format> -pipe`.
fn cli_flag(format: &str) -> Option<&'static str> {
    match format {
        "svg" => Some("-tsvg"),
        "preproc" => Some("-tpreproc"),
        "txt" | "atxt" => Some("-ttxt"),
        "utxt" => Some("-tutxt"),
        "xmi" | "xmi_star" | "xmi:star" => Some("-txmi:star"),
        "xmi_argo" | "xmi:argo" => Some("-txmi:argo"),
        "xmi_standard" | "xmi:standard" => Some("-txmi:standard"),
        "scxml" => Some("-tscxml"),
        "graphml" => Some("-tgraphml"),
        "latex" | "tex" => Some("-tlatex"),
        "debug" => Some("-tdebug"),
        _ => None,
    }
}

/// Renders `source` (PlantUML text) to the requested output format via Java
/// PlantUML.
///
/// Spawns `java -jar <jar> -t<format> -pipe` with `source` piped to stdin.
///
/// Returns `Some(output_string)` on success, or `None` if Java/jar is
/// unavailable or the command fails.
pub fn render(source: &str, format: &str) -> Option<String> {
    let flag = cli_flag(format)?;
    let jar = jar_path();
    if !jar.exists() {
        return None;
    }

    let mut child = Command::new("java")
        .arg("-jar")
        .arg(&jar)
        .arg(flag)
        .arg("-pipe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(source.as_bytes());
    }

    let output = child.wait_with_output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

/// Renders `source` (PlantUML text) to an SVG string via Java PlantUML.
///
/// Convenience wrapper around `render(source, "svg")`.
///
/// Returns `Some(svg_string)` on success, or `None` if Java/jar is unavailable
/// or the command fails.
pub fn render_svg(source: &str) -> Option<String> {
    render(source, "svg")
}
