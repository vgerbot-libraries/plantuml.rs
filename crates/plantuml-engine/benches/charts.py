#!/usr/bin/env python3
"""Render benchmark JSON results to SVG charts (stdlib only).

Reads the result directory produced by run.sh:

  <dir>/rust_steady.json  <dir>/java_steady.json
  <dir>/rust_once.json    <dir>/java_once.json   (optional)
  <dir>/cold.json         (optional)

Emits SVGs into <out-dir>:

  throughput.svg  steady-state per-render latency + single-pass wall time
  ratio_hist.svg  distribution of per-file java_ns / rust_ns ratios
  categories.svg  median ratio per vega corpus category
  scatter.svg     per-file rust vs java mean latency (log-log)
  cold_start.svg  cold-process wall time and peak RSS

plus summary.json with the headline numbers.

Ratio convention: java / rust — >1 means Rust is faster.

Usage: charts.py <result-dir> <out-dir>
Env (recorded in summary.json):
  BENCH_SHA   short commit SHA of the benchmarked tree
  BENCH_DATE  UTC date YYYY-MM-DD (defaults to today)
  BENCH_JAR   reference jar version label (defaults to 1.2026.6)
"""

import html
import json
import math
import os
import statistics
import sys
from datetime import datetime, timezone
from pathlib import Path

RUST = "#CE422B"
JAVA = "#5382A1"
JAVA_CLI = "#8C6D4B"
GRID = "#E5E7EB"
AXIS = "#374151"
TEXT = "#1F2937"
MUTED = "#6B7280"
FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def esc(s) -> str:
    return html.escape(str(s), quote=True)


# Filled in by main(); rendered small in the bottom-right of every chart.
META = ""


class Svg:
    """Tiny SVG canvas with a Cartesian plotting region."""

    def __init__(self, w, h, ml=60, mr=20, mt=40, mb=46, title=""):
        self.w, self.h = w, h
        self.ml, self.mr, self.mt, self.mb = ml, mr, mt, mb
        self.items = []
        if title:
            self.add(
                f'<text x="{w / 2}" y="24" text-anchor="middle" '
                f'font-size="14" font-weight="600" fill="{TEXT}">{esc(title)}</text>'
            )

    def add(self, s):
        self.items.append(s)

    @property
    def pw(self):
        return self.w - self.ml - self.mr

    @property
    def ph(self):
        return self.h - self.mt - self.mb

    def x(self, v, lo, hi):
        return self.ml + (v - lo) / (hi - lo) * self.pw

    def y(self, v, lo, hi):
        return self.mt + (1 - (v - lo) / (hi - lo)) * self.ph

    def text(self, x, y, s, **kw):
        if "anchor" in kw:
            kw["text-anchor"] = kw.pop("anchor")
        attrs = " ".join(f'{k}="{v}"' for k, v in kw.items())
        self.add(f'<text x="{x:.1f}" y="{y:.1f}" {attrs}>{esc(s)}</text>')

    def render(self):
        if META:
            self.add(
                f'<text x="{self.w - 12}" y="24" text-anchor="end" '
                f'font-size="10" fill="{MUTED}">{esc(META)}</text>'
            )
        return (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
            f'viewBox="0 0 {self.w} {self.h}" font-family="{FONT}">\n'
            + "\n".join(self.items)
            + "\n</svg>\n"
        )


def inline_legend(fig, entries, x, y):
    """Stacked legend with a translucent white backdrop, inside the plot."""
    w = max(len(label) for _, label in entries) * 7.0 + 36
    h = len(entries) * 18 + 10
    fig.add(f'<rect x="{x - 7}" y="{y - 15}" width="{w:.0f}" height="{h}" '
            f'fill="white" fill-opacity="0.85" rx="3"/>')
    for i, (color, label) in enumerate(entries):
        yy = y + i * 18
        fig.add(f'<rect x="{x}" y="{yy - 10}" width="12" height="12" '
                f'fill="{color}" rx="2"/>')
        fig.text(x + 17, yy, label, anchor="start",
                 **{"font-size": 11.5, "fill": TEXT})


def nice_ticks(lo, hi, n=5):
    if lo == hi:
        return [lo]
    step = (hi - lo) / max(1, n - 1)
    mag = 10 ** math.floor(math.log10(step))
    for m in (1, 2, 2.5, 5, 10):
        if step <= m * mag:
            step = m * mag
            break
    start = math.ceil(lo / step) * step
    out = []
    v = start
    while v < hi - step * 1e-9:
        out.append(round(v, 10))
        v += step
    # Top tick must cover hi so scaled geometry never leaves the plot.
    top = round(math.ceil(hi / step) * step, 10)
    if not out or out[-1] < top:
        out.append(top)
    return out


def fmt_num(v):
    if v >= 100:
        return f"{v:.0f}"
    if v >= 10:
        return f"{v:.1f}"
    if v >= 1:
        return f"{v:.2f}"
    return f"{v:.3f}"


def fmt_us(v):
    if v >= 1_000_000:
        return f"{v / 1_000_000:g}s"
    if v >= 1000:
        return f"{v / 1000:g}ms"
    return f"{v:g}"


def ratio_label(t):
    """Compact label for an octave tick: 2x, 1/2x, 1/8x, …"""
    if t >= 0:
        return f"{2 ** t:g}x"
    return f"1/{2 ** -t:g}x"


def load(d, name):
    p = d / name
    return json.loads(p.read_text()) if p.exists() else None


def category(path):
    parts = Path(path).parts
    if "vega" in parts:
        i = parts.index("vega")
        if i + 1 < len(parts) - 1:
            return parts[i + 1]
    return "(root)"


def join_files(rust, java):
    jr = {r["path"]: r for r in java["per_file"]}
    return [(r["path"], r, jr[r["path"]]) for r in rust["per_file"]
            if r["path"] in jr]


def paired_bars(fig, x0, panel_w, vals, names, colors, title,
                unit="", v_pad=1.18):
    """Draw one vertical bar-group panel; returns the base y."""
    vmax = max(vals) * v_pad
    ticks = nice_ticks(0, vmax, 5)
    vmax = ticks[-1]
    base_y = fig.mt + fig.ph - 34
    plot_h = fig.ph - 62
    gap = 26
    bw = min(110.0, (panel_w - gap - 20) / len(vals))
    total = len(vals) * bw + (len(vals) - 1) * gap
    x = x0 + (panel_w - total) / 2
    for t in ticks:
        yy = base_y - t / vmax * plot_h
        fig.add(f'<line x1="{x0:.1f}" y1="{yy:.1f}" x2="{x0 + panel_w:.1f}" '
                f'y2="{yy:.1f}" stroke="{GRID}"/>')
        fig.text(x0 - 8, yy + 4, fmt_num(t), anchor="end",
                 **{"font-size": 11, "fill": MUTED})
    for val, name, color in zip(vals, names, colors):
        bh = val / vmax * plot_h
        fig.add(f'<rect x="{x:.1f}" y="{base_y - bh:.1f}" width="{bw:.0f}" '
                f'height="{bh:.1f}" fill="{color}" rx="3"/>')
        fig.text(x + bw / 2, base_y - bh - 7, fmt_num(val) + unit,
                 anchor="middle", **{"font-size": 13, "font-weight": 600,
                                     "fill": TEXT})
        fig.text(x + bw / 2, base_y + 18, name, anchor="middle",
                 **{"font-size": 12, "fill": TEXT})
        x += bw + gap
    fig.text(x0 + panel_w / 2, base_y + 42, title, anchor="middle",
             **{"font-size": 12, "fill": MUTED})
    fig.add(f'<line x1="{x0:.1f}" y1="{base_y}" x2="{x0 + panel_w:.1f}" '
            f'y2="{base_y}" stroke="{AXIS}"/>')


def chart_throughput(out, rust, java, once_r, once_j):
    rt, jt = rust["total_ns"] / 1e9, java["total_ns"] / 1e9
    renders = rust["files"] * rust["iters"]
    r_us, j_us = rt / renders * 1e6, jt / renders * 1e6
    ratio = jt / rt

    fig = Svg(900, 330, title="Aggregate throughput — all corpus files")
    panels = [("Steady state per render", [r_us, j_us], ["Rust", "Java"],
               [RUST, JAVA])]
    if once_r and once_j:
        panels.append((
            "Single cold-cache pass, total wall (s)",
            [once_r["total_ns"] / 1e9, once_j["total_ns"] / 1e9],
            ["Rust", "Java"], [RUST, JAVA],
        ))
    gap = 70
    panel_w = (fig.pw - gap * (len(panels) - 1)) / len(panels)
    for idx, (title, vals, names, colors) in enumerate(panels):
        paired_bars(fig, fig.ml + idx * (panel_w + gap), panel_w,
                    vals, names, colors, title)
    fig.text(fig.ml + fig.pw / 2, fig.h - 10,
             f"Java / Rust total-time ratio: {ratio:.2f}x   (>1 means Rust is faster)",
             anchor="middle", **{"font-size": 13, "font-weight": 600,
                                 "fill": TEXT})
    (out / "throughput.svg").write_text(fig.render())
    return {"ratio": ratio, "r_us": r_us, "j_us": j_us, "renders": renders}


def chart_ratio_hist(out, ratios):
    logs = [math.log2(r) for r in ratios]
    lo, hi = min(logs), max(logs)
    blo = math.floor(lo * 4) / 4
    bhi = math.ceil(hi * 4) / 4
    if bhi == blo:
        bhi += 0.25
    step = 0.25
    edges = [blo + i * step
             for i in range(int(round((bhi - blo) / step)) + 1)]
    counts = [0] * (len(edges) - 1)
    for v in logs:
        k = min(int((v - blo) / step), len(counts) - 1)
        counts[k] += 1
    cmax = max(counts)

    fig = Svg(900, 340, title="Per-file latency ratio distribution (java / rust)")
    xticks = [e for e in edges if abs(e - round(e)) < 1e-9]
    yticks = nice_ticks(0, cmax, 5)
    fig.add(f'<rect x="{fig.ml}" y="{fig.mt}" width="{fig.pw}" '
            f'height="{fig.ph}" fill="white"/>')
    for t in yticks:
        yy = fig.y(t, 0, yticks[-1])
        fig.add(f'<line x1="{fig.ml}" y1="{yy:.1f}" x2="{fig.ml + fig.pw}" '
                f'y2="{yy:.1f}" stroke="{GRID}"/>')
        fig.text(fig.ml - 8, yy + 4, f"{t:g}", anchor="end",
                 **{"font-size": 11, "fill": MUTED})
    for t in xticks:
        xx = fig.x(t, blo, bhi)
        fig.add(f'<line x1="{xx:.1f}" y1="{fig.mt}" x2="{xx:.1f}" '
                f'y2="{fig.mt + fig.ph}" stroke="{GRID}"/>')
        fig.text(xx, fig.mt + fig.ph + 18, ratio_label(t), anchor="middle",
                 **{"font-size": 11, "fill": MUTED})
    bw = fig.pw / len(counts)
    for i, c in enumerate(counts):
        if c == 0:
            continue
        bh = c / yticks[-1] * fig.ph
        color = RUST if edges[i] >= 0 else JAVA
        fig.add(f'<rect x="{fig.ml + i * bw + 1:.1f}" '
                f'y="{fig.mt + fig.ph - bh:.1f}" '
                f'width="{max(bw - 2, 1):.1f}" height="{bh:.1f}" '
                f'fill="{color}" opacity="0.85"/>')
    xx0 = fig.x(0, blo, bhi)
    fig.add(f'<line x1="{xx0:.1f}" y1="{fig.mt}" x2="{xx0:.1f}" '
            f'y2="{fig.mt + fig.ph}" stroke="{AXIS}" stroke-width="1.5" '
            f'stroke-dasharray="4,3"/>')
    fig.add(f'<rect x="{fig.ml}" y="{fig.mt}" width="{fig.pw}" '
            f'height="{fig.ph}" fill="none" stroke="{AXIS}"/>')
    inline_legend(fig, [(RUST, "Rust faster (ratio > 1)"),
                        (JAVA, "Java faster (ratio < 1)")],
                  fig.ml + fig.pw - 215, fig.mt + 30)
    fig.text(fig.ml + fig.pw / 2, fig.h - 10,
             "java_ns / rust_ns per file", anchor="middle",
             **{"font-size": 12, "fill": MUTED})
    (out / "ratio_hist.svg").write_text(fig.render())


def chart_categories(out, by_cat):
    med = sorted(((statistics.median(v), c, len(v)) for c, v in by_cat.items()),
                 key=lambda t: t[0])
    n = len(med)
    rh = max(30 * n + 90, 180)
    fig = Svg(900, rh, ml=180, mr=70, mt=44, mb=46,
              title="Median latency ratio by corpus category")
    vmax = nice_ticks(0, max(m for m, _, _ in med) * 1.15, 6)[-1]
    bar_h = min(22, (fig.ph - 12) // max(n, 1))
    for i, (m, cat, cnt) in enumerate(med):
        yy = fig.mt + 8 + i * (bar_h + 6)
        bw = m / vmax * fig.pw
        color = RUST if m >= 1 else JAVA
        fig.add(f'<rect x="{fig.ml}" y="{yy:.1f}" width="{bw:.1f}" '
                f'height="{bar_h}" fill="{color}" opacity="0.88" rx="2"/>')
        fig.text(fig.ml - 10, yy + bar_h / 2 + 4, f"{cat} (n={cnt})",
                 anchor="end", **{"font-size": 11.5, "fill": TEXT})
        fig.text(fig.ml + bw + 6, yy + bar_h / 2 + 4, f"{m:.2f}x",
                 anchor="start", **{"font-size": 11.5, "font-weight": 600,
                                    "fill": TEXT})
    xx1 = fig.x(1, 0, vmax)
    fig.add(f'<line x1="{xx1:.1f}" y1="{fig.mt}" x2="{xx1:.1f}" '
            f'y2="{fig.mt + fig.ph}" stroke="{AXIS}" stroke-width="1.5" '
            f'stroke-dasharray="4,3"/>')
    for t in nice_ticks(0, vmax, 6):
        xx = fig.x(t, 0, vmax)
        fig.add(f'<line x1="{xx:.1f}" y1="{fig.mt + fig.ph}" x2="{xx:.1f}" '
                f'y2="{fig.mt + fig.ph + 5}" stroke="{AXIS}"/>')
        fig.text(xx, fig.mt + fig.ph + 20, f"{t:g}x", anchor="middle",
                 **{"font-size": 11, "fill": MUTED})
    fig.add(f'<line x1="{fig.ml}" y1="{fig.mt + fig.ph}" '
            f'x2="{fig.ml + fig.pw}" y2="{fig.mt + fig.ph}" stroke="{AXIS}"/>')
    (out / "categories.svg").write_text(fig.render())


def chart_scatter(out, rows):
    pts = [(r["mean_ns"] / 1e3, j["mean_ns"] / 1e3) for _, r, j in rows]
    allv = [v for p in pts for v in p]
    lo = 10 ** math.floor(math.log10(min(allv) * 0.8))
    hi = 10 ** math.ceil(math.log10(max(allv) * 1.25))
    llo, lhi = math.log10(lo), math.log10(hi)

    def lx(v):
        return fig.x(math.log10(v), llo, lhi)

    def ly(v):
        return fig.y(math.log10(v), llo, lhi)

    fig = Svg(760, 500, ml=72, mr=24, mt=40, mb=58,
              title="Per-file mean latency: Rust vs Java")
    ticks = [10 ** e for e in range(int(llo), int(lhi) + 1)]
    fig.add(f'<rect x="{fig.ml}" y="{fig.mt}" width="{fig.pw}" '
            f'height="{fig.ph}" fill="white"/>')
    for t in ticks:
        xx, yy = lx(t), ly(t)
        fig.add(f'<line x1="{xx:.1f}" y1="{fig.mt}" x2="{xx:.1f}" '
                f'y2="{fig.mt + fig.ph}" stroke="{GRID}"/>')
        fig.add(f'<line x1="{fig.ml}" y1="{yy:.1f}" x2="{fig.ml + fig.pw}" '
                f'y2="{yy:.1f}" stroke="{GRID}"/>')
        fig.text(xx, fig.mt + fig.ph + 18, fmt_us(t), anchor="middle",
                 **{"font-size": 11, "fill": MUTED})
        fig.text(fig.ml - 10, yy + 4, fmt_us(t), anchor="end",
                 **{"font-size": 11, "fill": MUTED})
    fig.add(f'<line x1="{lx(lo):.1f}" y1="{ly(lo):.1f}" x2="{lx(hi):.1f}" '
            f'y2="{ly(hi):.1f}" stroke="{AXIS}" stroke-width="1.5" '
            f'stroke-dasharray="5,4"/>')
    fig.text(lx(lo * 2.2), ly(lo * 2.2) + 14, "parity (1x)",
             **{"font-size": 11, "fill": MUTED,
                "stroke": "white", "stroke-width": 3,
                "paint-order": "stroke"})
    for rx, jy in pts:
        color = RUST if jy > rx else JAVA
        fig.add(f'<circle cx="{lx(rx):.1f}" cy="{ly(jy):.1f}" r="3.2" '
                f'fill="{color}" fill-opacity="0.65"/>')
    fig.add(f'<rect x="{fig.ml}" y="{fig.mt}" width="{fig.pw}" '
            f'height="{fig.ph}" fill="none" stroke="{AXIS}"/>')
    inline_legend(fig, [(RUST, "Rust faster"), (JAVA, "Java faster")],
                  fig.ml + fig.pw - 150, fig.mt + 30)
    fig.text(fig.ml + fig.pw / 2, fig.h - 12,
             "Rust mean latency per file (log scale)", anchor="middle",
             **{"font-size": 12, "fill": MUTED})
    fig.text(20, fig.mt + fig.ph / 2,
             "Java mean latency per file (log scale)", anchor="middle",
             **{"font-size": 12, "fill": MUTED,
                "transform": f"rotate(-90 20 {fig.mt + fig.ph / 2})"})
    (out / "scatter.svg").write_text(fig.render())


def chart_cold(out, cold):
    names = [n for n in ("rust", "java_bench", "java_cli") if n in cold]
    if not names:
        return
    labels = {"rust": "Rust", "java_bench": "Java harness",
              "java_cli": "java -jar CLI"}
    colors = {"rust": RUST, "java_bench": JAVA, "java_cli": JAVA_CLI}

    fig = Svg(900, 340, title="Cold process: one full corpus pass (startup included)")
    panels = [
        ("wall_s", "Wall time (s)", 1.0),
        ("rss_kb", "Peak RSS (MB)", 1024.0),
    ]
    gap = 70
    panel_w = (fig.pw - gap) / 2
    for pi, (key, title, scale) in enumerate(panels):
        x0 = fig.ml + pi * (panel_w + gap)
        vals = [cold[n][key] / scale for n in names]
        paired_bars(fig, x0, panel_w, vals,
                    [labels[n] for n in names],
                    [colors[n] for n in names], title, v_pad=1.2)
    (out / "cold_start.svg").write_text(fig.render())


def main(result_dir, out_dir):
    d = Path(result_dir)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)

    date = (os.environ.get("BENCH_DATE")
            or datetime.now(timezone.utc).strftime("%Y-%m-%d"))
    sha = os.environ.get("BENCH_SHA", "")
    jar = os.environ.get("BENCH_JAR", "1.2026.6")
    global META
    META = f"{date} · {sha[:8] + ' · ' if sha else ''}jar {jar}"

    rust = load(d, "rust_steady.json")
    java = load(d, "java_steady.json")
    if rust is None or java is None:
        sys.exit("missing rust_steady.json / java_steady.json")
    once_r = load(d, "rust_once.json")
    once_j = load(d, "java_once.json")
    cold = load(d, "cold.json")

    rows = join_files(rust, java)
    ratios = [j["mean_ns"] / r["mean_ns"] for _, r, j in rows]
    by_cat = {}
    for path, r, j in rows:
        by_cat.setdefault(category(path), []).append(j["mean_ns"] / r["mean_ns"])

    agg = chart_throughput(out, rust, java, once_r, once_j)
    chart_ratio_hist(out, ratios)
    chart_categories(out, by_cat)
    chart_scatter(out, rows)
    if cold:
        chart_cold(out, cold)

    n = len(ratios)
    summary = {
        "date": date,
        "sha": sha,
        "jar": jar,
        "n": n,
        "iters": rust["iters"],
        "warmup": rust["warmup"],
        "ratio_total": agg["ratio"],
        "ratio_geo_mean": math.exp(sum(math.log(x) for x in ratios) / n),
        "ratio_median": statistics.median(ratios),
        "win": sum(1 for x in ratios if x > 1),
        "rust_per_render_us": agg["r_us"],
        "java_per_render_us": agg["j_us"],
    }
    if once_r and once_j:
        summary["once_ratio"] = once_j["total_ns"] / once_r["total_ns"]
    if cold:
        summary["cold"] = cold
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: charts.py <result-dir> <out-dir>")
    main(sys.argv[1], sys.argv[2])
