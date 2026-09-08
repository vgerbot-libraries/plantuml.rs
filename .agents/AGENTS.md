# AGENTS.md

Guide for AI agents working on the plantuml.rs project.

## Project Overview

plantuml.rs is a 100% Rust reimplementation of [PlantUML](https://plantuml.com/), the Java diagram-generation library (~1500–1900 source files, 112 sub-packages). The goal is behavioral parity with the Java original, plus language bindings for Java, TypeScript, and Python via FFI.

## Reference Repo

`temp/plantuml/` contains the original Java source (shallow clone, gitignored). **Always read the corresponding Java file before porting.** See `.agents/rules/reference-usage.md` for key paths and usage guidelines.

## Workspace Structure

See `.agents/rules/workspace-conventions.md` for the cargo workspace layout, crate naming conventions (`plantuml-*` prefix), and the full list of planned crates.

## Porting Rules

See `.agents/rules/java-to-rust-porting.md` for:
- Naming conventions (PascalCase → snake_case for files and methods; package → crate path)
- File mapping (1:1 Java → Rust; file size governance ≤500/501–800/>800)
- OOP → Rust conversion (interfaces → traits, classes → structs, overloads → builders)
- Error handling (`Result` + `thiserror` for libraries, `anyhow` for CLI; no panics)
- Collections, null handling, concurrency mappings

## Architecture

See `.agents/architecture.md` for:
- The PlantUML pipeline (BlockUmlBuilder → BlockUml → PSystemBuilder → Diagram → export)
- Layer mapping (Java package → Rust crate)
- Design patterns to preserve (Factory+Registry, Command, Strategy, Lazy init, Template method)
- Rendering backends and binding strategy

## Installed Skills

The following skills are installed in `.agents/skills/`:

| Skill | Purpose |
|-------|---------|
| `rust-best-practices` | Rust coding standards: ownership, error handling, clippy, testing, generics, type state, docs, pointer safety |
| `rust-java-migration` | Java→Rust migration rules: directory path alignment, naming conversion (PascalCase → snake_case), 1:1 file mapping, layout governance |
| `rust-testing` | Rust testing patterns: TDD, unit/integration/property-based tests, benchmarks |
| `rust-ffi` | Rust FFI: bindgen, cbindgen, safe wrappers, sys crate structure, build.rs linking |

## Conventions

- **Rust edition**: 2021, `resolver = "2"`.
- **Error handling**: `thiserror` for library crates, `anyhow` for CLI/application. No panics in library code; use `Result`.
- **Doc comments**: all public items must have doc comments. Cite the Java source:
  ```rust
  /// Ported from: net/sourceforge/plantuml/SourceStringReader.java
  ```
- **Linting**: clippy clean; workspace-level lint config; deny warnings in CI.
- **Tests**: unit tests in `#[cfg(test)] mod tests`, integration tests in `tests/`.
- **No `unwrap()`/`expect()`** in library code except in test modules.

## Build Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Build the workspace |
| `cargo test` | Run all workspace tests |
| `cargo clippy --workspace -- -D warnings` | Lint the workspace (deny warnings) |

## Binding Builds

| Binding | Build Command |
|---------|-------------|
| Java | `cd bindings/plantuml-java && ./gradlew build` |
| Python | `maturin develop` (or `maturin build`) |
| TypeScript | `npm run build` |
