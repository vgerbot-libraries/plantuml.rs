#!/usr/bin/env bash
# Build both symmetric harnesses, classify the vega corpus on each side,
# intersect the successes, and run steady/once/cold measurements.
#
# Run from the repo root:
#   crates/plantuml-engine/benches/run.sh [result-dir]
# Env: ITERS=20 WARMUP=5 COLD_RUNS=3 CORPUS=... JAR=...
set -euo pipefail

HERE="crates/plantuml-engine/benches"
CORPUS="${CORPUS:-crates/plantuml-engine/tests/resources/vega}"
JAR="${JAR:-crates/plantuml-engine/tests/vendor/plantuml.jar}"
ITERS="${ITERS:-20}"
WARMUP="${WARMUP:-5}"
COLD_RUNS="${COLD_RUNS:-3}"
OUT="${1:-$HERE/results/last}"
JBUILD="$HERE/java/build"

mkdir -p "$OUT" "$JBUILD"

echo "== build rust bench (release) =="
cargo build --release --bench symbench --quiet
RS_BENCH="$(ls -t target/release/deps/symbench-* | grep -v '\.d$' | head -1)"
test -x "$RS_BENCH"

echo "== compile java harness =="
javac -cp "$JAR" -d "$JBUILD" "$HERE/java/SymBench.java" 2>&1 | grep -v 'warning:' || true
JAVA_CP="$JBUILD:$JAR"
# Pin both sides to one physical core when taskset is available.
PIN=""
if command -v taskset >/dev/null; then PIN="taskset -c 7"; fi
JVM_BASE="$PIN java -Xbatch -XX:+UseSerialGC"

echo "== classify corpus =="
$RS_BENCH manifest "$CORPUS" "$OUT/rust_all.tsv"
$PIN java -cp "$JAVA_CP" SymBench manifest "$CORPUS" "$OUT/java_all.tsv" </dev/null

# Intersection: files ok on both sides. Java "error diagrams" are already
# excluded by the Syntax Error check in each harness.
python3 - "$OUT" <<'PY'
import sys
out = sys.argv[1]
def load(name):
    d = {}
    for line in open(f"{out}/{name}"):
        p, s = line.rstrip("\n").split("\t")
        d[p] = s
    return d
r, j = load("rust_all.tsv"), load("java_all.tsv")
common = sorted(set(r) & set(j))
inter = [p for p in common if r[p] == "ok" and j[p] == "ok"]
with open(f"{out}/manifest.tsv", "w") as f:
    for p in inter:
        f.write(f"{p}\tok\n")
only_r = [p for p in common if r[p] == "ok" and j[p] != "ok"]
only_j = [p for p in common if j[p] == "ok" and r[p] != "ok"]
print(f"intersection ok: {len(inter)} | rust-only ok: {len(only_r)} | java-only ok: {len(only_j)}")
PY

echo "== steady-state: ${WARMUP} warmup passes + ${ITERS} timed passes =="
$RS_BENCH run "$OUT/manifest.tsv" "$ITERS" "$WARMUP" "$OUT/rust_steady.json"
$JVM_BASE -cp "$JAVA_CP" SymBench run "$OUT/manifest.tsv" "$ITERS" "$WARMUP" "$OUT/java_steady.json" </dev/null

echo "== single in-process pass (read+render) =="
$RS_BENCH once "$OUT/manifest.tsv" "$OUT/rust_once.json"
$PIN java -cp "$JAVA_CP" SymBench once "$OUT/manifest.tsv" "$OUT/java_once.json" </dev/null

echo "== cold processes: ${COLD_RUNS} runs each, min wall / median RSS =="
cut -f1 "$OUT/manifest.tsv" > "$OUT/files.txt"
# This jar's --null still writes *.null next to inputs; redirect all output.
COLD_DIR="$OUT/coldout"
mkdir -p "$COLD_DIR"
run_cold() {
    local label="$1"; shift
    local best="" rss_lines=()
    for i in $(seq 1 "$COLD_RUNS"); do
        timeout 120 /usr/bin/time -v "$@" </dev/null >/dev/null 2>"$OUT/.time.$label.$i" || true
        local wall rss secs
        wall=$(awk -F': ' '/Elapsed \(wall/{print $2}' "$OUT/.time.$label.$i")
        rss=$(awk -F': ' '/Maximum resident set size/{print $2}' "$OUT/.time.$label.$i")
        [ -z "$wall" ] || [ -z "$rss" ] && { echo "$label: missing time output" >&2; continue; }
        rss_lines+=("$rss")
        secs=$(echo "$wall" | awk -F'[:.]' '{printf "%.3f", ($1*60)+$2+"0."$3}')
        best=$(echo "$best $secs" | awk '{if ($1=="" || $2<$1) print $2; else print $1}')
    done
    local rss_med
    rss_med=$(printf '%s\n' "${rss_lines[@]}" | sort -n | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
    [ -n "$best" ] && echo "$label $best $rss_med"
}

{
    run_cold rust "$RS_BENCH" null $(tr '\n' ' ' < "$OUT/files.txt")
    run_cold java_bench $PIN java -cp "$JAVA_CP" SymBench null $(tr '\n' ' ' < "$OUT/files.txt")
    run_cold java_cli $PIN java -jar "$JAR" -o "$COLD_DIR" --null $(tr '\n' ' ' < "$OUT/files.txt")
} | python3 -c '
import json, sys
d = {}
for line in sys.stdin:
    parts = line.split()
    if len(parts) != 3:
        continue
    name, wall, rss = parts
    d[name] = {"wall_s": float(wall), "rss_kb": int(rss)}
json.dump(d, open(sys.argv[1], "w"), indent=2)
' "$OUT/cold.json"
rm -f "$OUT"/.time.* "$OUT/files.txt"

echo
python3 "$HERE/compare.py" "$OUT" | tee "$OUT/REPORT.md"
