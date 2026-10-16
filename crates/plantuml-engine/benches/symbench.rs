//! Symmetric cross-language benchmark harness (Rust side).
//!
//! Paired with `benches/java/SymBench.java`: both implement the same
//! subcommands and emit the same TSV/JSON formats so results are directly
//! comparable.
//!
//! Boundary under test: source text -> SVG string, i.e.
//! [`plantuml_engine::render`] with [`FileFormat::Svg`], the Rust counterpart of
//! Java's `new SourceStringReader(defines, source)` + `outputImage(sink, SVG)`.
//!
//! Subcommands:
//! - `manifest <corpus-dir> <out.tsv>`: classify every `*.puml` as ok/fail.
//! - `run <manifest.tsv> <iters> <warmup> <out.json>`: steady-state latency.
//! - `once <manifest.tsv> <out.json>`: one timed pass over the manifest.
//! - `null <file>...`: render each file once, discard output (cold-process mode).

use std::fs;
use std::hint::black_box;
use std::io::{self, Write};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use plantuml_core::FileFormat;

/// Render outcome recorded in a manifest row.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    Fail,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fail => "fail",
        }
    }
}

fn render_ok(src: &str) -> bool {
    let r = std::panic::catch_unwind(|| plantuml_engine::render(src, FileFormat::Svg));
    matches!(r, Ok(Ok(svg)) if svg.contains("<svg"))
}

fn walk_puml(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read corpus dir") {
        let path = entry.expect("read dir entry").path();
        if path.is_dir() {
            walk_puml(&path, out);
        } else if path.extension().is_some_and(|e| e == "puml") {
            out.push(path);
        }
    }
}

fn write_manifest(corpus: &Path, dest: &Path) -> io::Result<()> {
    let mut files = Vec::new();
    walk_puml(corpus, &mut files);
    files.sort();

    let mut w = io::BufWriter::new(fs::File::create(dest)?);
    let mut ok = 0usize;
    for f in &files {
        let src = fs::read_to_string(f)?;
        let status = if render_ok(&src) {
            ok += 1;
            Status::Ok
        } else {
            Status::Fail
        };
        writeln!(w, "{}\t{}", f.display(), status.as_str())?;
    }
    eprintln!("manifest: {ok}/{} ok", files.len());
    Ok(())
}

struct Row {
    path: String,
    status: Status,
}

fn read_manifest(path: &Path) -> Vec<Row> {
    let text = fs::read_to_string(path).expect("read manifest");
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let (path, status) = line.split_once('\t').expect("manifest row: path<TAB>status");
            Row {
                path: path.to_owned(),
                status: if status == "ok" {
                    Status::Ok
                } else {
                    Status::Fail
                },
            }
        })
        .collect()
}

/// Render one source; returns elapsed nanoseconds and whether it succeeded.
fn timed_render(src: &str) -> (u128, bool) {
    let start = Instant::now();
    let r = std::panic::catch_unwind(|| plantuml_engine::render(src, FileFormat::Svg));
    let ns = start.elapsed().as_nanos();
    let svg = match &r {
        Ok(Ok(svg)) => Some(svg),
        _ => None,
    };
    let ok = svg.is_some_and(|s| s.contains("<svg"));
    if let Some(svg) = svg {
        black_box(svg.as_bytes());
    }
    (ns, ok)
}

/// Steady-state: `warmup` untimed passes, then `iters` timed passes, over the
/// ok rows of the manifest. File order is identical on both sides so JIT/cache
/// effects align; interleaving files within each pass keeps measurement noise
/// shared rather than assigning it to a single diagram.
fn run_steady(manifest: &Path, iters: u32, warmup: u32, dest: &Path) -> io::Result<()> {
    let rows: Vec<Row> = read_manifest(manifest)
        .into_iter()
        .filter(|r| r.status == Status::Ok)
        .collect();
    let sources: Vec<String> = rows
        .iter()
        .map(|r| fs::read_to_string(&r.path).expect("read corpus file"))
        .collect();

    for _ in 0..warmup {
        for src in &sources {
            let (_, ok) = timed_render(src);
            assert!(ok, "manifest-ok file failed during warmup");
        }
    }

    let mut sums = vec![0u128; sources.len()];
    let mut mins = vec![u128::MAX; sources.len()];
    let mut fails = 0u64;
    let global = Instant::now();
    for _ in 0..iters {
        for (i, src) in sources.iter().enumerate() {
            let (ns, ok) = timed_render(src);
            if !ok {
                fails += 1;
            }
            sums[i] += ns;
            mins[i] = mins[i].min(ns);
        }
    }
    let total_ns = global.elapsed().as_nanos();

    let mut w = io::BufWriter::new(fs::File::create(dest)?);
    writeln!(w, "{{")?;
    writeln!(w, r#"  "side": "rust","#)?;
    writeln!(w, r#"  "mode": "steady","#)?;
    writeln!(w, r#"  "files": {},"#, sources.len())?;
    writeln!(w, r#"  "iters": {iters},"#)?;
    writeln!(w, r#"  "warmup": {warmup},"#)?;
    writeln!(w, r#"  "fails": {fails},"#)?;
    writeln!(w, r#"  "total_ns": {total_ns},"#)?;
    writeln!(w, r#"  "per_file": ["#)?;
    for (i, row) in rows.iter().enumerate() {
        let comma = if i + 1 == rows.len() { "" } else { "," };
        let mean = sums[i] / u128::from(iters);
        let path = json_escape(&row.path);
        writeln!(
            w,
            r#"    {{"path": "{path}", "mean_ns": {mean}, "min_ns": {}}}{comma}"#,
            mins[i]
        )?;
    }
    writeln!(w, "  ]")?;
    writeln!(w, "}}")?;
    Ok(())
}

/// Minimal JSON string escaper; avoids pulling serde into the bench binary.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// One pass: reads from disk like the CLI path would; reports total wall time.
fn run_once(manifest: &Path, dest: &Path) -> io::Result<()> {
    let rows = read_manifest(manifest);
    let global = Instant::now();
    let mut rendered = 0u64;
    let mut fails = 0u64;
    for row in &rows {
        if row.status != Status::Ok {
            continue;
        }
        let src = fs::read_to_string(&row.path)?;
        let (_, ok) = timed_render(&src);
        if ok {
            rendered += 1;
        } else {
            fails += 1;
        }
    }
    let total_ns = global.elapsed().as_nanos();
    let mut w = io::BufWriter::new(fs::File::create(dest)?);
    writeln!(
        w,
        r#"{{"side": "rust", "mode": "once", "files": {rendered}, "fails": {fails}, "total_ns": {total_ns}}}"#
    )?;
    Ok(())
}

/// Cold-process mode: render each argv file once, discard output. Timed
/// externally by `/usr/bin/time`; this binary only does the work.
fn run_null(files: &[String]) -> io::Result<()> {
    let mut sink = io::BufWriter::new(io::sink());
    for f in files {
        let src = fs::read_to_string(f)?;
        if let Ok(svg) = plantuml_engine::render(&src, FileFormat::Svg) {
            sink.write_all(svg.as_bytes())?;
        }
    }
    sink.flush()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("manifest") => {
            assert_eq!(args.len(), 3, "usage: manifest <corpus-dir> <out.tsv>");
            write_manifest(Path::new(&args[1]), Path::new(&args[2])).unwrap();
        }
        Some("run") => {
            assert_eq!(
                args.len(),
                5,
                "usage: run <manifest.tsv> <iters> <warmup> <out.json>"
            );
            run_steady(
                Path::new(&args[1]),
                args[2].parse().unwrap(),
                args[3].parse().unwrap(),
                Path::new(&args[4]),
            )
            .unwrap();
        }
        Some("once") => {
            assert_eq!(args.len(), 3, "usage: once <manifest.tsv> <out.json>");
            run_once(Path::new(&args[1]), Path::new(&args[2])).unwrap();
        }
        Some("null") => run_null(&args[1..]).unwrap(),
        _ => {
            eprintln!(
                "usage: symbench {{manifest <dir> <out.tsv> | run <m.tsv> <iters> <warmup> <out.json> | once <m.tsv> <out.json> | null <file>...}}"
            );
            std::process::exit(2);
        }
    }
}
