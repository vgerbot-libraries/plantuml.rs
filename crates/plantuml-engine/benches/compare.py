#!/usr/bin/env python3
"""Compare Rust vs Java steady-state results emitted by the symmetric harness.

Reads two JSON files produced by `symbench run` (same manifest, same iters),
joins per-file samples, and prints a Markdown performance report. Also folds in
once-pass totals and cold-process measurements when present in the result dir:

  <dir>/rust_steady.json  <dir>/java_steady.json
  <dir>/rust_once.json    <dir>/java_once.json   (optional)
  <dir>/cold.json         (optional)

Ratio convention: java_ns / rust_ns — >1 means Rust is faster.
"""

import json
import math
import statistics
import sys
from pathlib import Path


def percentile(xs, p):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    k = (len(xs) - 1) * p
    lo = math.floor(k)
    hi = math.ceil(k)
    if lo == hi:
        return xs[lo]
    return xs[lo] * (hi - k) + xs[hi] * (k - lo)


def category(path: str) -> str:
    parts = Path(path).parts
    if "vega" in parts:
        i = parts.index("vega")
        if i + 1 < len(parts) - 1:
            return parts[i + 1]
    return "(root)"


def load(path: Path):
    with path.open() as f:
        return json.load(f)


def main(result_dir: str) -> None:
    d = Path(result_dir)
    rust = load(d / "rust_steady.json")
    java = load(d / "java_steady.json")

    assert rust["files"] == java["files"], "manifest file counts differ"
    assert rust["iters"] == java["iters"], "iter counts differ"

    jr = {row["path"]: row for row in java["per_file"]}
    ratios_mean = []
    ratios_min = []
    by_cat: dict[str, list[float]] = {}
    rows_out = []
    for r in rust["per_file"]:
        j = jr.get(r["path"])
        if j is None:
            continue
        ratio_mean = j["mean_ns"] / r["mean_ns"]
        ratio_min = j["min_ns"] / r["min_ns"]
        ratios_mean.append(ratio_mean)
        ratios_min.append(ratio_min)
        by_cat.setdefault(category(r["path"]), []).append(ratio_mean)
        rows_out.append((ratio_mean, r["path"], r["mean_ns"], j["mean_ns"]))

    n = len(ratios_mean)
    geo_mean = math.exp(sum(math.log(x) for x in ratios_mean) / n)

    print("# PlantUML.rs vs PlantUML Java — SVG render benchmark\n")
    print(f"- files compared (intersection): **{n}**")
    print(f"- timed iters/file: {rust['iters']} (warmup {rust['warmup']})")
    print(f"- JVM: Corretto/OpenJDK 17, jar 1.2026.6; Rust: release profile")
    print(f"- timing fails recorded: rust={rust['fails']} java={java['fails']}")
    print("- ratio = java_ns / rust_ns (**>1 → Rust faster**)\n")

    # Aggregate wall time across the whole pass: total_ns includes loop overhead,
    # but both harnesses execute the identical loop, so it is the apples-to-apples
    # throughput number.
    rt, jt = rust["total_ns"], java["total_ns"]
    print("## Aggregate throughput (all files, timed passes)\n")
    print("| side | total | per render |")
    print("|---|---:|---:|")
    renders = n * rust["iters"]
    print(f"| Rust | {rt/1e9:.3f} s | {rt/renders/1e3:.1f} µs |")
    print(f"| Java | {jt/1e9:.3f} s | {jt/renders/1e3:.1f} µs |")
    print(f"\n**Java/Rust total-time ratio: {jt/rt:.2f}x**\n")

    print("## Per-file latency ratio (java_mean / rust_mean)\n")
    print("| statistic | value |")
    print("|---|---:|")
    print(f"| geometric mean | {geo_mean:.2f}x |")
    print(f"| median | {statistics.median(ratios_mean):.2f}x |")
    print(f"| p10 | {percentile(ratios_mean, 0.10):.2f}x |")
    print(f"| p90 | {percentile(ratios_mean, 0.90):.2f}x |")
    print(f"| best case for Rust (min-time ratio) median | {statistics.median(ratios_min):.2f}x |")
    win = sum(1 for x in ratios_mean if x > 1)
    print(f"| files where Rust is faster | {win}/{n} ({win/n:.0%}) |\n")

    print("## By corpus category (median ratio, n)\n")
    print("| category | n | median java/rust |")
    print("|---|---:|---:|")
    for cat in sorted(by_cat, key=lambda c: statistics.median(by_cat[c])):
        xs = by_cat[cat]
        print(f"| {cat} | {len(xs)} | {statistics.median(xs):.2f}x |")

    print("\n## Slowest files (Rust mean latency)\n")
    print("| file | rust µs | java µs | ratio |")
    print("|---|---:|---:|---:|")
    for _, path, r_ns, j_ns in sorted(rows_out, key=lambda x: -x[2])[:10]:
        rel = path.split("vega/", 1)[-1]
        print(f"| `{rel}` | {r_ns/1e3:.1f} | {j_ns/1e3:.1f} | {j_ns/r_ns:.2f}x |")
    print("\n## Biggest Rust regressions (lowest java/rust ratio)\n")
    print("| file | rust µs | java µs | ratio |")
    print("|---|---:|---:|---:|")
    for ratio, path, r_ns, j_ns in sorted(rows_out, key=lambda x: x[0])[:10]:
        rel = path.split("vega/", 1)[-1]
        print(f"| `{rel}` | {r_ns/1e3:.1f} | {j_ns/1e3:.1f} | {ratio:.2f}x |")

    ro = d / "rust_once.json"
    jo = d / "java_once.json"
    if ro.exists() and jo.exists():
        r1, j1 = load(ro), load(jo)
        print("\n## Single cold-cache pass (read + render, in-process)\n")
        print("| side | files | total wall |")
        print("|---|---:|---:|")
        print(f"| Rust | {r1['files']} | {r1['total_ns']/1e9:.3f} s |")
        print(f"| Java | {j1['files']} | {j1['total_ns']/1e9:.3f} s |")
        print(f"\nratio: **{j1['total_ns']/r1['total_ns']:.2f}x** "
              "(first pass: JVM interpreted/C1, Rust native)\n")

    cold = d / "cold.json"
    if cold.exists():
        c = load(cold)
        print("## Cold process (startup + one full pass)\n")
        print("| invocation | wall (s) | max RSS (MB) |")
        print("|---|---:|---:|")
        for name, m in c.items():
            print(f"| {name} | {m['wall_s']:.3f} | {m['rss_kb']/1024:.0f} |")
        print()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: compare.py <result-dir>")
    main(sys.argv[1])
