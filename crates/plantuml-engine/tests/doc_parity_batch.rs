//! Batch comparison of ALL language-doc examples: Java PlantUML vs plantuml.rs.
//!
//! Unlike `java_parity_test.rs`, this never asserts. It renders every
//! `.puml` file in `temp/doc-parity/src/` with both implementations,
//! writes raw + cleaned SVGs to `temp/doc-parity/out/`, and emits a
//! machine-readable `report.json` plus a human-readable `report.txt`.
//!
//! Run with:
//!   cargo test -p plantuml-engine --test doc_parity_batch -- --nocapture

mod java_plantuml;
mod svg_cleaner;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

#[derive(Debug)]
struct CaseReport {
    name: String,
    status: String,
    rust_error: Option<String>,
    java_available: bool,
    java_dims: Option<[String; 2]>,
    rust_dims: Option<[String; 2]>,
    cleaned_lines_java: usize,
    cleaned_lines_rust: usize,
    /// First differing line pairs (line index, java line, rust line).
    first_diffs: Vec<DiffLine>,
    /// Number of diff hunks from the LCS line diff.
    diff_hunks: usize,
}

#[derive(Debug)]
struct DiffLine {
    line: usize,
    java: String,
    rust: String,
}

impl CaseReport {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "status": self.status,
            "rust_error": self.rust_error,
            "java_available": self.java_available,
            "java_dims": self.java_dims.as_ref().map(|d| json!([d[0], d[1]])),
            "rust_dims": self.rust_dims.as_ref().map(|d| json!([d[0], d[1]])),
            "cleaned_lines_java": self.cleaned_lines_java,
            "cleaned_lines_rust": self.cleaned_lines_rust,
            "first_diffs": self.first_diffs.iter().map(|d| json!({
                "line": d.line, "java": d.java, "rust": d.rust
            })).collect::<Vec<_>>(),
            "diff_hunks": self.diff_hunks,
        })
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn root_svg_dims(svg: &str) -> Option<[String; 2]> {
    // Find the first <svg ...> opening tag.
    let start = svg.find("<svg")?;
    let rest = &svg[start..];
    let end = rest.find('>')?;
    let tag = &rest[..end];
    let get = |attr: &str| -> Option<String> {
        let key = format!("{attr}=\"");
        let pos = tag.find(&key)? + key.len();
        let val = &tag[pos..];
        let stop = val.find('"')?;
        Some(val[..stop].to_string())
    };
    let w = get("width")?;
    let h = get("height")?;
    Some([w, h])
}

fn first_line_diffs(java: &[&str], rust: &[&str], limit: usize) -> Vec<DiffLine> {
    let mut out = Vec::new();
    for (i, (j, r)) in java.iter().zip(rust.iter()).enumerate() {
        if j != r {
            out.push(DiffLine {
                line: i,
                java: j.trim().to_string(),
                rust: r.trim().to_string(),
            });
            if out.len() >= limit {
                return out;
            }
        }
    }
    out
}

/// Count changed-line hunks via a simple LCS line diff.
fn count_hunks(java: &[&str], rust: &[&str]) -> usize {
    let n = java.len();
    let m = rust.len();
    // LCS DP table (bounded guard).
    let mut dp = vec![vec![0u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if java[i] == rust[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    // Walk the table to collect edit ops; count contiguous hunks.
    let mut hunks = 0;
    let mut in_hunk = false;
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if java[i] == rust[j] {
            i += 1;
            j += 1;
            in_hunk = false;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            if !in_hunk {
                hunks += 1;
                in_hunk = true;
            }
            i += 1; // deletion from java
        } else {
            if !in_hunk {
                hunks += 1;
                in_hunk = true;
            }
            j += 1; // insertion in rust
        }
    }
    if i < n || j < m {
        if !in_hunk {
            hunks += 1;
        }
    }
    hunks
}

#[test]
fn batch_compare() {
    unsafe {
        std::env::set_var("PLANTUML_NO_DOT", "1");
    }

    // TEMP DEBUG: log panics (caught by catch_unwind below) with backtrace.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("RUST PANIC: {info}\n{}", std::backtrace::Backtrace::force_capture());
    }));

    let root = workspace_root();
    let src_dir = root.join("temp/doc-parity/src");
    let out_dir = root.join("temp/doc-parity/out");
    let java_dir = out_dir.join("java");
    let rust_dir = out_dir.join("rust");
    let clean_java_dir = out_dir.join("cleaned-java");
    let clean_rust_dir = out_dir.join("cleaned-rust");
    for d in [
        &java_dir,
        &rust_dir,
        &clean_java_dir,
        &clean_rust_dir,
    ] {
        fs::create_dir_all(d).unwrap();
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(&src_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "puml"))
        .collect();
    paths.sort();

    let mut cases: Vec<CaseReport> = Vec::new();
    for path in &paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(path).unwrap();

        // Java
        let java_svg_opt = java_plantuml::render_svg(&source);
        let java_available = java_svg_opt.is_some();

        // Rust — catch both errors and unwinding panics so one bad case
        // cannot abort the whole batch.
        let rust_result = std::panic::catch_unwind(|| plantuml_engine::render_svg(&source))
            .unwrap_or_else(|payload| {
                let msg = payload
                    .downcast_ref::<&'static str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_string());
                Err(plantuml_engine::RenderError::Export(format!("Rust panic: {msg}")))
            });

        let mut case = CaseReport {
            name: name.clone(),
            status: String::new(),
            rust_error: None,
            java_available,
            java_dims: None,
            rust_dims: None,
            cleaned_lines_java: 0,
            cleaned_lines_rust: 0,
            first_diffs: Vec::new(),
            diff_hunks: 0,
        };

        if let Some(java_svg) = &java_svg_opt {
            fs::write(java_dir.join(format!("{name}.svg")), java_svg).unwrap();
            case.java_dims = root_svg_dims(java_svg);
        }

        match &rust_result {
            Ok(rust_svg) => {
                fs::write(rust_dir.join(format!("{name}.svg")), rust_svg).unwrap();
                case.rust_dims = root_svg_dims(rust_svg);
            }
            Err(e) => {
                case.rust_error = Some(format!("{e:?}"));
            }
        }

        match (&java_svg_opt, &rust_result) {
            (Some(java_svg), Ok(rust_svg)) => {
                let cj = svg_cleaner::clean(java_svg);
                let cr = svg_cleaner::clean(rust_svg);
                fs::write(clean_java_dir.join(format!("{name}.svg")), &cj).unwrap();
                fs::write(clean_rust_dir.join(format!("{name}.svg")), &cr).unwrap();
                let jl: Vec<&str> = cj.lines().collect();
                let rl: Vec<&str> = cr.lines().collect();
                case.cleaned_lines_java = jl.len();
                case.cleaned_lines_rust = rl.len();
                if cj == cr {
                    case.status = "MATCH".to_string();
                } else {
                    case.status = "MISMATCH".to_string();
                    case.first_diffs = first_line_diffs(&jl, &rl, 8);
                    case.diff_hunks = count_hunks(&jl, &rl);
                }
            }
            (None, Ok(_)) => case.status = "JAVA_UNAVAILABLE".to_string(),
            (Some(_), Err(_)) => case.status = "RUST_ERROR".to_string(),
            (None, Err(_)) => case.status = "BOTH_FAILED".to_string(),
        }

        cases.push(case);
    }

    // ── report.json ──
    let json_values: Vec<Value> = cases.iter().map(|c| c.to_json()).collect();
    let json = serde_json::to_string_pretty(&json_values).unwrap();
    fs::write(out_dir.join("report.json"), &json).unwrap();

    // ── report.txt ──
    let mut by_status: BTreeMap<&str, Vec<&CaseReport>> = BTreeMap::new();
    for c in &cases {
        by_status.entry(c.status.as_str()).or_default().push(c);
    }
    let mut txt = String::new();
    txt.push_str(&format!("Total examples: {}\n\n", cases.len()));
    for (status, group) in &by_status {
        txt.push_str(&format!("{status}: {}\n", group.len()));
    }
    txt.push('\n');
    for c in &cases {
        txt.push_str(&format!(
            "{:<16} {:<16}",
            c.name, c.status,
        ));
        if let (Some(j), Some(r)) = (&c.java_dims, &c.rust_dims) {
            txt.push_str(&format!(" java={}x{} rust={}x{}", j[0], j[1], r[0], r[1]));
        }
        if c.status == "MISMATCH" {
            txt.push_str(&format!(
                " lines(java/rust)={}/{} hunks={}",
                c.cleaned_lines_java, c.cleaned_lines_rust, c.diff_hunks
            ));
        }
        if let Some(e) = &c.rust_error {
            txt.push_str(&format!(" error={e}"));
        }
        txt.push('\n');
    }
    fs::write(out_dir.join("report.txt"), &txt).unwrap();
    println!("\n{txt}");
}
