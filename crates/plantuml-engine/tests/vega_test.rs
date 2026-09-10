//! Vega test harness — walks .puml files and runs PREPROC tests.
//!
//! Ported from `test.vega.VegaTest` and `test.vega.VegaInputFile`.

mod svg_cleaner;
mod yaml_parser;

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use plantuml_core::{FileFormat, FileFormatOption};
use plantuml_preproc::preproc::Defines;
use plantuml_engine::SourceStringReader;

use yaml_parser::VegaYaml;

/// The vega resources directory.
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
        if !yaml_done && line.trim() == "---" {
            if !inside_yaml {
                inside_yaml = true;
                continue;
            } else {
                inside_yaml = false;
                yaml_done = true;
                continue;
            }
        }

        if inside_yaml {
            yaml_lines.push(line.to_string());
        } else {
            puml_lines.push(line.to_string());
        }
    }

    let yaml = if yaml_lines.is_empty() {
        // Default: allow failure, output svg
        let mut y = VegaYaml::default();
        y.output = Some("svg".to_string());
        y.allow_failure = true;
        y
    } else {
        VegaYaml::parse(&yaml_lines)
    };

    (yaml, puml_lines.join("\n"))
}

/// Normalizes line endings to `\n`.
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Returns the expected output file path for the given puml path and extension.
fn expected_file(puml_path: &Path, extension: &str) -> PathBuf {
    let name = puml_path.file_name().unwrap().to_string_lossy();
    let base_name = name.rsplit_once('.').map_or(name.as_ref(), |(base, _)| base);
    puml_path.with_file_name(format!("{base_name}{extension}"))
}

/// Runs a single PREPROC test.
fn run_preproc_test(puml_path: &Path) {
    let (yaml, source) = parse_puml_file(puml_path);

    // Skip if not PREPROC format
    let output_format = yaml.output.as_deref().unwrap_or("svg");
    if output_format != "preproc" {
        return;
    }

    // Create SourceStringReader and output PREPROC
    let current_dir = puml_path.parent().map(std::path::Path::to_path_buf);
    let ssr = SourceStringReader::with_defines_and_dir(&source, &Defines::new(), current_dir);
    let file_format_option = FileFormatOption::new(FileFormat::Preproc);
    let mut output = Vec::new();
    let description = ssr.output_image(&mut Cursor::new(&mut output), 0, &file_format_option);

    // Verify output was generated
    assert!(
        description.is_some(),
        "No output generated for {}",
        puml_path.display()
    );

    let actual_output = normalize_line_endings(&String::from_utf8_lossy(&output));

    // Compare against expected .preproc file
    let expected_path = expected_file(puml_path, ".preproc");
    let expected_output = match fs::read_to_string(&expected_path) {
        Ok(content) => normalize_line_endings(&content),
        Err(_) => {
            // If expected file doesn't exist, write it (for initial generation)
            fs::write(&expected_path, &actual_output).ok();
            return;
        }
    };

    assert_eq!(
        expected_output, actual_output,
        "PREPROC output mismatch for {}",
        puml_path.display()
    );
}

#[test]
fn test_all_preproc_vega_files() {
    let vega_dir = vega_resources();
    let mut preproc_files = Vec::new();

    // Walk all .puml files and find PREPROC ones
    walk_puml_files(&vega_dir, &mut preproc_files);

    assert!(!preproc_files.is_empty(), "No PREPROC .puml files found");

    for puml_path in &preproc_files {
        let (yaml, _) = parse_puml_file(puml_path);
        if yaml.allow_failure {
            // Skip known failures
            eprintln!("Skipping (allow-failure): {}", puml_path.display());
            continue;
        }
        run_preproc_test(puml_path);
    }
}

/// Recursively walks .puml files that have corresponding .preproc files.
fn walk_puml_files(dir: &Path, preproc_files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_puml_files(&path, preproc_files);
        } else if path.extension().is_some_and(|ext| ext == "puml") {
            // Check if this file has a .preproc expected file
            let preproc_path = expected_file(&path, ".preproc");
            if preproc_path.exists() {
                preproc_files.push(path);
            }
        }
    }
}

#[test]
fn test_preproc_eval() {
    let path = vega_resources().join("preproc").join("eval.puml");
    run_preproc_test(&path);
}

#[test]
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
fn test_self_001() {
    run_sequence_svg_test("asciiverse/self_001");
}

#[test]
fn test_hello_002() {
    run_sequence_svg_test("asciiverse/hello_002");
}

#[test]
fn test_multiline_001() {
    run_sequence_svg_test("asciiverse/multiline_001");
}

#[test]
fn test_selfnote_003() {
    run_sequence_svg_test("asciiverse/selfnote_003");
}

#[test]
fn test_group_001() {
    run_sequence_svg_test("asciiverse/group_001");
}

#[test]
fn test_mvp_hello_both() {
    run_sequence_svg_test("mvp/hello-both");
}

#[test]
fn test_altpar_001() {
    run_sequence_svg_test("asciiverse/altpar_001");
}

#[test]
fn test_altpar_002() {
    run_sequence_svg_test("asciiverse/altpar_002");
}

#[test]
fn test_altpar_003() {
    run_sequence_svg_test("asciiverse/altpar_003");
}

#[test]
fn test_altpar_004() {
    run_sequence_svg_test("asciiverse/altpar_004");
}

#[test]
fn test_altpar_005() {
    run_sequence_svg_test("asciiverse/altpar_005");
}

#[test]
fn test_altpar_006() {
    run_sequence_svg_test("asciiverse/altpar_006");
}

#[test]
fn test_altpar_007() {
    run_sequence_svg_test("asciiverse/altpar_007");
}

#[test]
fn test_altpar_008() {
    run_sequence_svg_test("asciiverse/altpar_008");
}

#[test]
fn test_basic_002() {
    run_sequence_svg_test("asciiverse/basic_002");
}

#[test]
fn test_basic_003() {
    run_sequence_svg_test("asciiverse/basic_003");
}

#[test]
fn test_layout_001() {
    run_sequence_svg_test("asciiverse/layout_001");
}

#[test]
fn test_layout_002() {
    run_sequence_svg_test("asciiverse/layout_002");
}

#[test]
fn test_layout_002b() {
    run_sequence_svg_test("asciiverse/layout_002b");
}

#[test]
fn test_layout_003() {
    run_sequence_svg_test("asciiverse/layout_003");
}

#[test]
fn test_leftmsg_001() {
    run_sequence_svg_test("asciiverse/leftmsg_001");
}

#[test]
fn test_leftmsg_002() {
    run_sequence_svg_test("asciiverse/leftmsg_002");
}

#[test]
fn test_leftmsg_003() {
    run_sequence_svg_test("asciiverse/leftmsg_003");
}

#[test]
fn test_nested_001() {
    run_sequence_svg_test("asciiverse/nested_001");
}

#[test]
fn test_partition_001() {
    run_sequence_svg_test("asciiverse/partition_001");
}

#[test]
fn test_selfnote_001() {
    run_sequence_svg_test("asciiverse/selfnote_001");
}

#[test]
fn test_selfnote_001b() {
    run_sequence_svg_test("asciiverse/selfnote_001b");
}

#[test]
fn test_selfnote_001c() {
    run_sequence_svg_test("asciiverse/selfnote_001c");
}

#[test]
fn test_selfnote_002() {
    run_sequence_svg_test("asciiverse/selfnote_002");
}

#[test]
fn test_timeline_001() {
    run_sequence_svg_test("asciiverse/timeline_001");
}

#[test]
fn test_timeline_002() {
    run_sequence_svg_test("asciiverse/timeline_002");
}


/// Runs a sequence diagram SVG test: parses the .puml, renders SVG, compares against expected .svg.
fn run_sequence_svg_test(name: &str) {
    let puml_path = vega_resources().join(format!("{name}.puml"));
    let expected_path = vega_resources().join(format!("{name}.svg"));

    let puml_content = fs::read_to_string(&puml_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", puml_path.display()));
    let expected_svg = fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", expected_path.display()));

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
    );
    let cleaned_actual = svg_cleaner::clean(&actual_svg);
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
                eprintln!("Line {}: expected: {}", i, e);
                eprintln!("Line {}: actual:   {}", i, a);
            }
        }
        if exp_lines.len() != act_lines.len() {
            eprintln!("Line count: expected={}, actual={}", exp_lines.len(), act_lines.len());
        }
    }

    assert_eq!(cleaned_actual, cleaned_expected, "{name} SVG mismatch");
}
