#!/usr/bin/env bash
#
# prepare.sh — One-shot development setup for the plantuml.rs site
#
# Prepares everything needed to run `npm run dev` inside site/:
#   1. Verify prerequisites (Node, npm, wasm-pack, wasm32-unknown-unknown target)
#   2. Install deps & build the @vgerbot/plantuml TypeScript binding (incl. WASM)
#   3. Install site npm dependencies
#   4. Sync WASM artifacts into site/public/wasm/
#
# Usage:
#   ./prepare.sh
#
# After it succeeds: npm run dev   (or: npm run build / npm run preview)

set -euo pipefail

# ─── Colors ──────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
  BOLD=$'\033[1m'; BLUE=$'\033[34m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; RED=$'\033[31m'; NC=$'\033[0m'
else
  BOLD=''; BLUE=''; GREEN=''; YELLOW=''; RED=''; NC=''
fi

# ─── Paths ───────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TS_BINDING="$PROJECT_ROOT/bindings/plantuml-ts"

# ─── Logging ─────────────────────────────────────────────────────────────────
log_info() { printf "${BLUE}[INFO]${NC}  %s\n" "$*"; }
log_ok()   { printf "${GREEN}[OK]${NC}    %s\n" "$*"; }
log_warn() { printf "${YELLOW}[WARN]${NC}  %s\n" "$*"; }
log_err()  { printf "${RED}[ERROR]${NC} %s\n" "$*" >&2; }
log_step() { printf "\n${BOLD}═══ %s ═══${NC}\n" "$*"; }

has() { command -v "$1" &>/dev/null; }

# ─── Prerequisites ───────────────────────────────────────────────────────────
check_prerequisites() {
  log_step "Prerequisite check"

  local missing=0

  if has node; then
    log_ok "node $(node --version)"
  else
    log_err "node not found. Install Node.js $(cat "$PROJECT_ROOT/.nvmrc") from https://nodejs.org"
    missing=1
  fi

  if has npm; then
    log_ok "npm $(npm --version)"
  else
    log_err "npm not found."
    missing=1
  fi

  if has wasm-pack; then
    log_ok "wasm-pack $(wasm-pack --version 2>/dev/null | head -1)"
  else
    log_err "wasm-pack not found. Install with: cargo install wasm-pack"
    missing=1
  fi

  if has rustup && rustup target list --installed 2>/dev/null | grep -q "wasm32-unknown-unknown"; then
    log_ok "wasm32-unknown-unknown target installed"
  else
    log_err "Rust wasm32 target not found. Install with: rustup target add wasm32-unknown-unknown"
    missing=1
  fi

  if [[ $missing -ne 0 ]]; then
    exit 1
  fi
}

# ─── TypeScript binding (includes WASM build) ───────────────────────────────
prepare_binding() {
  log_step "TypeScript binding (@vgerbot/plantuml)"

  log_info "Installing binding dependencies (npm install) …"
  (cd "$TS_BINDING" && npm install)

  log_info "Building binding (wasm-pack web + nodejs, tsc) …"
  (cd "$TS_BINDING" && npm run build)

  log_ok "Binding built → $TS_BINDING/dist"
}

# ─── Site ────────────────────────────────────────────────────────────────────
prepare_site() {
  log_step "Site (Astro + Starlight)"

  log_info "Installing site dependencies (npm install) …"
  (cd "$SCRIPT_DIR" && npm install)

  log_info "Syncing WASM into public/wasm/ …"
  (cd "$SCRIPT_DIR" && npm run sync:wasm)

  log_ok "Site ready"
}

# ─── Main ────────────────────────────────────────────────────────────────────
main() {
  check_prerequisites
  prepare_binding
  prepare_site

  printf "\n${GREEN}${BOLD}Preparation complete.${NC} Next:\n"
  printf "  cd site && npm run dev\n"
}

main
