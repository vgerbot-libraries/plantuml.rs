#!/usr/bin/env bash
# Copy vega test fixtures from the Java reference into Rust crate test directories.
#
# Usage: ./scripts/copy_test_fixtures.sh [diagram_type...]
# If no arguments, copies all available fixture types.
#
# Prerequisites: temp/plantuml/ must contain the Java source clone.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
VEGA="$ROOT_DIR/temp/plantuml/src/test/resources/vega"

if [[ ! -d "$VEGA" ]]; then
    echo "Error: vega test resources not found at $VEGA"
    echo "Ensure temp/plantuml/ contains the Java source clone."
    exit 1
fi

copy_fixtures() {
    local name="$1"
    local src="$2"
    local dest="$3"
    if [[ ! -d "$src" ]]; then
        echo "  SKIP $name: source $src not found"
        return 0
    fi
    mkdir -p "$dest"
    cp -r "$src"/* "$dest/" 2>/dev/null || true
    echo "  OK   $name: $src → $dest"
}

copy_glob() {
    local name="$1"
    local pattern="$2"
    local dest="$3"
    local count
    count=$(ls $pattern 2>/dev/null | wc -l)
    if [[ "$count" -eq 0 ]]; then
        echo "  SKIP $name: no files matching $pattern"
        return 0
    fi
    mkdir -p "$dest"
    cp $pattern "$dest/" 2>/dev/null || true
    echo "  OK   $name: $pattern → $dest ($count files)"
}

ALL_TYPES="state timing xgantt wbs mindmap class component description activity"

TYPES="${*:-$ALL_TYPES}"

echo "Copying test fixtures for: $TYPES"
echo "Source: $VEGA"
echo ""

for type in $TYPES; do
    case "$type" in
        state)
            copy_fixtures "state" "$VEGA/state" "$ROOT_DIR/crates/plantuml-state/tests/fixtures"
            ;;
        timing)
            copy_fixtures "timing" "$VEGA/timing" "$ROOT_DIR/crates/plantuml-timing/tests/fixtures"
            ;;
        xgantt|gantt)
            copy_fixtures "gantt" "$VEGA/xgantt" "$ROOT_DIR/crates/plantuml-gantt/tests/fixtures"
            ;;
        wbs)
            copy_fixtures "wbs" "$VEGA/wbs" "$ROOT_DIR/crates/plantuml-mindmap/tests/fixtures/wbs"
            copy_fixtures "wbs-nonreg" "$VEGA/nonreg/wbs" "$ROOT_DIR/crates/plantuml-mindmap/tests/fixtures/wbs_nonreg"
            ;;
        mindmap)
            copy_fixtures "mindmap" "$VEGA/nonreg/mindmap" "$ROOT_DIR/crates/plantuml-mindmap/tests/fixtures/mindmap_nonreg"
            copy_glob "mindmap-svg-id" "$VEGA/svg/id/mindmap_*.puml" "$ROOT_DIR/crates/plantuml-mindmap/tests/fixtures"
            ;;
        class)
            copy_fixtures "class-xmi" "$VEGA/xmi/clazz" "$ROOT_DIR/crates/plantuml-class/tests/fixtures"
            copy_glob "class-nonreg" "$VEGA/nonreg/group2814/*.puml" "$ROOT_DIR/crates/plantuml-class/tests/fixtures"
            copy_glob "class-nonreg-svg" "$VEGA/nonreg/group2814/*.svg" "$ROOT_DIR/crates/plantuml-class/tests/fixtures"
            ;;
        component|description)
            copy_glob "component-svg-id" "$VEGA/svg/id/component_*.puml" "$ROOT_DIR/crates/plantuml-description/tests/fixtures"
            copy_fixtures "component-xmi" "$VEGA/xmi/component" "$ROOT_DIR/crates/plantuml-description/tests/fixtures"
            copy_glob "component-nonreg" "$VEGA/nonreg/simple/ComponentExtraArrows_0001.puml" "$ROOT_DIR/crates/plantuml-description/tests/fixtures"
            ;;
        activity)
            echo "  NOTE: activity test cases are scattered across vega/nonreg/ groups"
            echo "  Activity fixtures will be copied manually in Phase 9."
            ;;
        *)
            echo "  UNKNOWN type: $type"
            ;;
    esac
done

echo ""
echo "Done. Fixtures copied to respective crate test directories."
