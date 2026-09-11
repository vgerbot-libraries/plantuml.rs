#!/usr/bin/env bash
#
# init.sh — One-shot project initialization for plantuml.rs
#
# Installs all prerequisites and builds every artifact:
#   1. Rust toolchain + wasm32 target + wasm-pack
#   2. Rust workspace crates
#   3. WASM bindings (web + node targets via wasm-pack)
#   4. TypeScript binding package (@vgerbot/plantuml)
#   5. Site npm dependencies + WASM sync
#
# Usage:
#   ./scripts/init.sh              # run all phases
#   ./scripts/init.sh --rust       # only Rust toolchain + crates
#   ./scripts/init.sh --wasm      # only WASM build
#   ./scripts/init.sh --ts        # only TypeScript binding build
#   ./scripts/init.sh --site      # only site deps + wasm sync
#   ./scripts/init.sh --check     # verify prerequisites, don't install/build
#
# Flags can be combined: ./scripts/init.sh --rust --wasm
#
# Environment variables:
#   SKIP_RUST_INSTALL=1  Skip rustup/wasm-pack installation (assume present)
#

set -euo pipefail

# ─── Colors ──────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
    BOLD='\033[1m'; RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'
    BLUE='\033[0;34m'; NC='\033[0m'
else
    BOLD=''; RED=''; GREEN=''; YELLOW=''; BLUE=''; NC=''
fi

# ─── Paths ───────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_CRATE="$PROJECT_ROOT/crates/plantuml-wasm"
TS_BINDING="$PROJECT_ROOT/bindings/plantuml-ts"
SITE_DIR="$PROJECT_ROOT/site"

# ─── Logging ─────────────────────────────────────────────────────────────────
log_info()  { printf "${BLUE}[INFO]${NC}  %s\n" "$*"; }
log_ok()    { printf "${GREEN}[OK]${NC}    %s\n" "$*"; }
log_warn()  { printf "${YELLOW}[WARN]${NC}  %s\n" "$*"; }
log_err()   { printf "${RED}[ERROR]${NC} %s\n" "$*" >&2; }
log_step()  { printf "\n${BOLD}═══ %s ═══${NC}\n" "$*"; }

# ─── Phase selection ─────────────────────────────────────────────────────────
RUN_RUST=false
RUN_WASM=false
RUN_TS=false
RUN_SITE=false
CHECK_ONLY=false
NO_FLAGS=true

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rust)  RUN_RUST=true;  NO_FLAGS=false ;;
        --wasm)  RUN_WASM=true;  NO_FLAGS=false ;;
        --ts)    RUN_TS=true;    NO_FLAGS=false ;;
        --site)  RUN_SITE=true;  NO_FLAGS=false ;;
        --check) CHECK_ONLY=true; NO_FLAGS=false ;;
        --help|-h)
            sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            log_err "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

if $NO_FLAGS; then
    RUN_RUST=true
    RUN_WASM=true
    RUN_TS=true
    RUN_SITE=true
fi

# ─── Helpers ─────────────────────────────────────────────────────────────────
has() { command -v "$1" &>/dev/null; }

ensure_rust_target() {
    local target="wasm32-unknown-unknown"
    if rustup target list --installed 2>/dev/null | grep -q "$target"; then
        log_ok "Rust target $target already installed"
    else
        log_info "Adding Rust target $target …"
        rustup target add "$target"
        log_ok "Rust target $target installed"
    fi
}

ensure_wasm_pack() {
    if has wasm-pack; then
        log_ok "wasm-pack already installed ($(wasm-pack --version 2>/dev/null | head -1))"
    elif [[ -f "$HOME/.cargo/bin/wasm-pack" ]]; then
        log_ok "wasm-pack already installed"
    else
        log_info "Installing wasm-pack …"
        curl --proto '=https' --tlsv1.2 -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
        log_ok "wasm-pack installed"
    fi
}

# ─── Phase: Rust toolchain + crates ──────────────────────────────────────────
phase_rust() {
    log_step "Rust toolchain & crates"

    if ! has rustc || ! has cargo; then
        if [[ "${SKIP_RUST_INSTALL:-0}" == "1" ]]; then
            log_err "rustc/cargo not found and SKIP_RUST_INSTALL=1 — cannot continue"
            exit 1
        fi
        log_info "Installing Rust via rustup …"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
        log_ok "Rust installed ($(rustc --version))"
    else
        log_ok "Rust present ($(rustc --version))"
    fi

    ensure_rust_target

    log_info "Building Rust workspace (debug) …"
    (cd "$PROJECT_ROOT" && cargo build)
    log_ok "Rust workspace built"

    log_info "Running clippy …"
    (cd "$PROJECT_ROOT" && cargo clippy --workspace -- -D warnings)
    log_ok "Clippy clean"
}

# ─── Phase: WASM build ───────────────────────────────────────────────────────
phase_wasm() {
    log_step "WASM bindings"

    ensure_wasm_pack

    local wasm_out_web="$TS_BINDING/wasm/web"
    local wasm_out_node="$TS_BINDING/wasm/node"

    log_info "Building WASM (web target) …"
    wasm-pack build --target web "$WASM_CRATE" --out-dir "$wasm_out_web"
    # wasm-pack generates .gitignore files that would exclude the built artifacts
    rm -f "$wasm_out_web/.gitignore"
    log_ok "WASM web build → $wasm_out_web"

    log_info "Building WASM (nodejs target) …"
    wasm-pack build --target nodejs "$WASM_CRATE" --out-dir "$wasm_out_node"
    rm -f "$wasm_out_node/.gitignore"
    log_ok "WASM node build → $wasm_out_node"
}

# ─── Phase: TypeScript binding ───────────────────────────────────────────────
phase_ts() {
    log_step "TypeScript binding (@vgerbot/plantuml)"

    log_info "Installing npm dependencies …"
    (cd "$TS_BINDING" && npm install)
    log_ok "npm dependencies installed"

    log_info "Building TypeScript binding …"
    (cd "$TS_BINDING" && npm run build)
    log_ok "TypeScript binding built → $TS_BINDING/dist"
}

# ─── Phase: Site ─────────────────────────────────────────────────────────────
phase_site() {
    log_step "Site (Astro + Starlight)"

    log_info "Installing site npm dependencies …"
    (cd "$SITE_DIR" && npm install)
    log_ok "Site dependencies installed"

    log_info "Syncing WASM into site/public/wasm/ …"
    (cd "$SITE_DIR" && npm run sync:wasm)
    log_ok "WASM synced to site/public/wasm/"
}

# ─── Phase: Check ────────────────────────────────────────────────────────────
phase_check() {
    log_step "Prerequisite check"

    local missing=0

    printf "  %-20s " "rustc"
    if has rustc; then printf "${GREEN}✓${NC} %s\n" "$(rustc --version)"; else printf "${RED}✗ missing${NC}\n"; missing=1; fi

    printf "  %-20s " "cargo"
    if has cargo; then printf "${GREEN}✓${NC} %s\n" "$(cargo --version)"; else printf "${RED}✗ missing${NC}\n"; missing=1; fi

    printf "  %-20s " "wasm-pack"
    if has wasm-pack; then printf "${GREEN}✓${NC} %s\n" "$(wasm-pack --version 2>/dev/null | head -1)"; else printf "${YELLOW}⚠ not found (will install)${NC}\n"; fi

    printf "  %-20s " "wasm32 target"
    if rustup target list --installed 2>/dev/null | grep -q "wasm32-unknown-unknown"; then printf "${GREEN}✓${NC} installed\n"; else printf "${YELLOW}⚠ not found (will install)${NC}\n"; fi

    printf "  %-20s " "node"
    if has node; then printf "${GREEN}✓${NC} %s\n" "$(node --version)"; else printf "${RED}✗ missing${NC}\n"; missing=1; fi

    printf "  %-20s " "npm"
    if has npm; then printf "${GREEN}✓${NC} %s\n" "$(npm --version)"; else printf "${RED}✗ missing${NC}\n"; missing=1; fi

    if [[ $missing -eq 1 ]]; then
        echo ""
        log_err "Missing prerequisites. Install Rust from https://rustup.rs and Node.js from https://nodejs.org"
        exit 1
    fi

    echo ""
    log_ok "All core prerequisites present"
}

# ─── Main ────────────────────────────────────────────────────────────────────
main() {
    cd "$PROJECT_ROOT"

    if $CHECK_ONLY; then
        phase_check
        exit 0
    fi

    # Always run a quick check first (non-fatal for optional tools)
    phase_check

    if $RUN_RUST;  then phase_rust;  fi
    if $RUN_WASM;  then phase_wasm;  fi
    if $RUN_TS;    then phase_ts;    fi
    if $RUN_SITE;  then phase_site;  fi

    echo ""
    log_step "Done"
    log_ok "Project initialized successfully."
    echo ""
    log_info "Next steps:"
    printf "  • Run the site:  ${BOLD}cd site && npm run dev${NC}\n"
    printf "  • Run tests:     ${BOLD}cargo test${NC}\n"
    printf "  • Build site:    ${BOLD}cd site && npm run build${NC}\n"
}

main "$@"
