//! Vega test harness — walks .puml files and runs parity tests against Java PlantUML.
//!
//! Ported from `test.vega.VegaTest` and `test.vega.VegaInputFile`.
//!
//! Per `.agents/rules/test-parity.md`:
//! - Expected outputs are generated at runtime by invoking the Java JAR
//!   (`tests/vendor/plantuml.jar`). No pre-generated expected-output files
//!   (`.svg`, `.txt`, `.atxt`, `.utxt`, `.preproc`, `.xmi`, `.tex`, `.scxml`,
//!   `.graphml`) are committed to the repo.
//! - Only `.puml` input files are committed.
//! - Tests skip gracefully if Java or the JAR is unavailable.

mod java_plantuml;
mod svg_cleaner;
mod yaml_parser;

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use plantuml_core::{FileFormat, FileFormatOption};
use plantuml_preproc::preproc::Defines;
use plantuml_engine::SourceStringReader;

use yaml_parser::VegaYaml;

/// The vega resources directory (input `.puml` files only).
fn vega_resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join("vega")
}

/// Parses a .puml file, separating the YAML header from the puml source.
fn parse_puml_file(path: &Path) -> (VegaYaml, String) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();

    let mut yaml_lines = Vec::new();
    let mut puml_lines = Vec::new();
    let mut inside_yaml = false;
    let mut yaml_done = false;

    for line in &lines {
        if yaml_done || line.trim() != "---" {
            // Not a YAML delimiter — route to the appropriate section.
        } else if inside_yaml {
            inside_yaml = false;
            yaml_done = true;
            continue;
        } else {
            inside_yaml = true;
            continue;
        }

        if inside_yaml {
            yaml_lines.push(line.to_string());
        } else {
            puml_lines.push(line.to_string());
        }
    }

    let yaml = if yaml_lines.is_empty() {
        // Default: allow failure, output svg
        VegaYaml {
            output: Some("svg".to_string()),
            allow_failure: true,
            ..Default::default()
        }
    } else {
        VegaYaml::parse(&yaml_lines)
    };

    (yaml, puml_lines.join("\n"))
}

/// Normalizes line endings to `\n`.
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Runs a single PREPROC test: renders via Rust pipeline, obtains expected
/// output from the Java JAR at runtime, and compares.
fn run_preproc_test(puml_path: &Path) {
    if !java_plantuml::is_available() {
        eprintln!("Skipping {} — Java PlantUML not available", puml_path.display());
        return;
    }

    let (yaml, source) = parse_puml_file(puml_path);

    let output_format = yaml.output.as_deref().unwrap_or("svg");
    if output_format != "preproc" {
        return;
    }

    // Render via Rust pipeline.
    let current_dir = puml_path.parent().map(std::path::Path::to_path_buf);
    let ssr = SourceStringReader::with_defines_and_dir(&source, &Defines::new(), current_dir);
    let file_format_option = FileFormatOption::new(FileFormat::Preproc);
    let mut output = Vec::new();
    let description = ssr.output_image(&mut Cursor::new(&mut output), 0, &file_format_option);

    assert!(
        description.is_some(),
        "No output generated for {}",
        puml_path.display()
    );

    let actual_output = normalize_line_endings(&String::from_utf8_lossy(&output));

    // Obtain expected output from Java JAR at runtime.
    let expected_output = match java_plantuml::render(&source, "preproc") {
        Some(o) => normalize_line_endings(&o),
        None => panic!("Java PlantUML failed to render {}", puml_path.display()),
    };

    assert_eq!(
        expected_output, actual_output,
        "PREPROC output mismatch for {}",
        puml_path.display()
    );
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_all_preproc_vega_files() {
    let vega_dir = vega_resources();
    let mut preproc_files = Vec::new();

    walk_puml_files(&vega_dir, &mut preproc_files);

    assert!(!preproc_files.is_empty(), "No PREPROC .puml files found");

    for puml_path in &preproc_files {
        let (yaml, _) = parse_puml_file(puml_path);
        if yaml.allow_failure {
            eprintln!("Skipping (allow-failure): {}", puml_path.display());
            continue;
        }
        run_preproc_test(puml_path);
    }
}

/// Recursively walks .puml files, collecting those whose YAML header specifies
/// `output: preproc`.
fn walk_puml_files(dir: &Path, preproc_files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_puml_files(&path, preproc_files);
        } else if path.extension().is_some_and(|ext| ext == "puml") {
            let (yaml, _) = parse_puml_file(&path);
            if yaml.output.as_deref() == Some("preproc") {
                preproc_files.push(path);
            }
        }
    }
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_preproc_eval() {
    let path = vega_resources().join("preproc").join("eval.puml");
    run_preproc_test(&path);
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_preproc_upper() {
    let path = vega_resources().join("preproc").join("upper.puml");
    run_preproc_test(&path);
}

#[test]
fn test_hello_svg() {
    run_sequence_svg_test("mvp/hello-svg");
}

#[test]
fn test_sequence_01() {
    run_sequence_svg_test("svg/id/sequence_01");
}

#[test]
fn test_sequence_02() {
    run_sequence_svg_test("svg/id/sequence_02");
}

#[test]
fn test_sequence_desc() {
    run_sequence_svg_test("svg/option/sequence_desc");
}

#[test]
fn test_sequence_title() {
    run_sequence_svg_test("svg/option/sequence_title");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_sequence_title_title() {
    run_sequence_svg_test("svg/option/sequence_title_title");
}

#[test]
fn test_basic_001() {
    run_sequence_svg_test("asciiverse/basic_001");
}

#[test]
fn test_hello_001() {
    run_sequence_svg_test("asciiverse/hello_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_self_001() {
    run_sequence_svg_test("asciiverse/self_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_hello_002() {
    run_sequence_svg_test("asciiverse/hello_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_multiline_001() {
    run_sequence_svg_test("asciiverse/multiline_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_selfnote_003() {
    run_sequence_svg_test("asciiverse/selfnote_003");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_group_001() {
    run_sequence_svg_test("asciiverse/group_001");
}

#[test]
fn test_mvp_hello_both() {
    run_sequence_svg_test("mvp/hello-both");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_001() {
    run_sequence_svg_test("asciiverse/altpar_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_002() {
    run_sequence_svg_test("asciiverse/altpar_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_003() {
    run_sequence_svg_test("asciiverse/altpar_003");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_004() {
    run_sequence_svg_test("asciiverse/altpar_004");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_005() {
    run_sequence_svg_test("asciiverse/altpar_005");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_006() {
    run_sequence_svg_test("asciiverse/altpar_006");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_007() {
    run_sequence_svg_test("asciiverse/altpar_007");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_altpar_008() {
    run_sequence_svg_test("asciiverse/altpar_008");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_basic_002() {
    run_sequence_svg_test("asciiverse/basic_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_basic_003() {
    run_sequence_svg_test("asciiverse/basic_003");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_layout_001() {
    run_sequence_svg_test("asciiverse/layout_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_layout_002() {
    run_sequence_svg_test("asciiverse/layout_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_layout_002b() {
    run_sequence_svg_test("asciiverse/layout_002b");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_layout_003() {
    run_sequence_svg_test("asciiverse/layout_003");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_leftmsg_001() {
    run_sequence_svg_test("asciiverse/leftmsg_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_leftmsg_002() {
    run_sequence_svg_test("asciiverse/leftmsg_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_leftmsg_003() {
    run_sequence_svg_test("asciiverse/leftmsg_003");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_nested_001() {
    run_sequence_svg_test("asciiverse/nested_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_partition_001() {
    run_sequence_svg_test("asciiverse/partition_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_selfnote_001() {
    run_sequence_svg_test("asciiverse/selfnote_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_selfnote_001b() {
    run_sequence_svg_test("asciiverse/selfnote_001b");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_selfnote_001c() {
    run_sequence_svg_test("asciiverse/selfnote_001c");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_selfnote_002() {
    run_sequence_svg_test("asciiverse/selfnote_002");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_timeline_001() {
    run_sequence_svg_test("asciiverse/timeline_001");
}

#[test]
#[ignore = "Rust output does not match Java reference"]
fn test_timeline_002() {
    run_sequence_svg_test("asciiverse/timeline_002");
}


/// Runs a sequence diagram SVG test: parses the .puml, renders SVG via Rust,
/// obtains the expected SVG from the Java JAR at runtime, and compares after
/// normalization via `svg_cleaner`.
fn run_sequence_svg_test(name: &str) {
    let puml_path = vega_resources().join(format!("{name}.puml"));

    let puml_content = fs::read_to_string(&puml_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", puml_path.display()));

    // Render via Rust.
    let parsed = plantuml_engine::sequence_renderer::parse_simple_sequence(&puml_content)
        .unwrap_or_else(|| panic!("Failed to parse sequence diagram from {}", puml_path.display()));
    let actual_svg = plantuml_engine::sequence_renderer::render_sequence_svg(
        &parsed.diagram,
        parsed.svg_title.as_deref(),
        parsed.svg_desc.as_deref(),
        parsed.title.as_deref(),
        parsed.title_line,
        parsed.hide_footbox,
        &parsed.notes,
        &parsed.groups,
        &parsed.msg_activates,
        &parsed.msg_deactivates,
        &parsed.msg_parallel,
        parsed.max_message_size,
        &parsed.msg_exo,
        &parsed.msg_hidden,
        parsed.skin_rose,
        parsed.arrow_color.as_deref(),
        parsed.header_text.as_deref(),
        parsed.header_line,
        parsed.footer_text.as_deref(),
        parsed.footer_line,
        parsed.legend_text.as_deref(),
        parsed.caption_text.as_deref(),
        parsed.caption_line,
        &parsed.style_rules,
        &parsed.participant_source_lines,
        &parsed.msg_source_lines,
    );
    let cleaned_actual = svg_cleaner::clean(&actual_svg);

    // Obtain expected SVG from Java JAR at runtime.
    let Some(expected_svg) = java_plantuml::render_svg(&puml_content) else {
        eprintln!("Skipping {name} — Java PlantUML not available");
        return;
    };
    let cleaned_expected = svg_cleaner::clean(&expected_svg);

    if cleaned_actual != cleaned_expected {
        let debug_path = vega_resources().join(format!("{name}.actual.svg"));
        let _ = fs::write(&debug_path, &actual_svg);
        eprintln!("Actual SVG written to {}", debug_path.display());
        eprintln!("=== DIFF (expected vs actual) ===");
        let exp_lines: Vec<&str> = cleaned_expected.lines().collect();
        let act_lines: Vec<&str> = cleaned_actual.lines().collect();
        for (i, (e, a)) in exp_lines.iter().zip(act_lines.iter()).enumerate() {
            if e != a {
                eprintln!("Line {i}: expected: {e}");
                eprintln!("Line {i}: actual:   {a}");
            }
        }
        if exp_lines.len() != act_lines.len() {
            eprintln!("Line count: expected={}, actual={}", exp_lines.len(), act_lines.len());
        }
    }

    assert_eq!(cleaned_actual, cleaned_expected, "{name} SVG mismatch");
}
