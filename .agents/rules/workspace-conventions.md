# Workspace Conventions

Rules for the cargo workspace structure of plantuml.rs.

## Workspace Root

- `Cargo.toml` at the project root contains the `[workspace]` section.
- `resolver = "2"` (the modern resolver).
- Shared dependencies go in `[workspace.dependencies]` so all crates reference the same versions.
- Each crate's `Cargo.toml` references workspace deps via `dep.workspace = true`.

## Crate Naming

- All crates are prefixed with `plantuml-` (e.g., `plantuml-core`, `plantuml-klimt`).
- Crate names use `kebab-case` (hyphens), the cargo convention.
- Library crate names map to Rust identifiers by replacing hyphens with underscores (e.g., `plantuml-core` → `plantuml_core`).

## Crate Layout

```
crates/<name>/
├── Cargo.toml
├── src/
│   ├── lib.rs        # library crate root (or main.rs for binaries)
│   └── ...
└── tests/            # integration tests (optional)
```

## Planned Crates

Documented for future implementation. Not created in the bootstrap phase.

| Crate | Java Package(s) | Responsibility |
|-------|------------------|---------------|
| `plantuml-core` | root (partial) | `Diagram`, `TextBlock`, `StringBounder`, `FileFormat`, `FileFormatOption` traits/types |
| `plantuml-klimt` | `klimt` | 2D graphics abstraction: shapes, geometry, fonts, drawing surfaces, `UGraphic`, `TextBlock` |
| `plantuml-regex` | `com.plantuml.ubrex` | Custom regex engine port |
| `plantuml-preproc` | `preproc`, `tim` | Preprocessor: `!include`, `!define`, variables, conditionals |
| `plantuml-command` | `command` | `Command` / `CommandFactory` parsing framework; `BlocLines` wraps `StringLocated` from `plantuml-preproc` |
| `plantuml-model` | `abel`, `cucadiagram` | Entity/relationship model: `Entity`, `Link`, `LeafType` |
| `plantuml-sequence` | `sequencediagram` | Sequence diagram (first diagram type to implement) |
| `plantuml-class` | `classdiagram` | Class diagram |
| `plantuml-activity` | `activitydiagram3` | Activity diagram |
| `plantuml-skin` | `skin`, `style`, `theme` | Skin / style / theme system |
| `plantuml-layout` | `svek`, `elk`, `sdot` | Layout engines (Graphviz / ELK integration) |
| `plantuml-svg` | `svg` | SVG rendering backend |
| `plantuml-png` | `png` | PNG rendering backend |
| `plantuml-pdf` | `openpdf` | PDF rendering backend |
| `plantuml-engine` | root (partial) | Top-level pipeline: `BlockUml`, `BlockUmlBuilder`, `PSystemBuilder` |
| `plantuml-cli` | `Run`, `cli` | CLI binary (ports `Run.java`) |
| `plantuml-ffi` | — | C FFI shared library for language bindings |
| `bindings/plantuml-java` | — | Java bindings (JNI) |
| `bindings/plantuml-python` | — | Python bindings (PyO3 + maturin) |
| `bindings/plantuml-ts` | — | TypeScript bindings (napi-rs) |

## Dependency Policy

- Prefer pure-Rust dependencies. No C dependencies unless unavoidable.
- Shared dependencies declared once in `[workspace.dependencies]`.
- Each crate inherits via `dep.workspace = true` to keep versions in sync.
- Evaluate each dependency for maintenance status, license compatibility (MIT), and purity (no C transitive deps).

## Edition

- Rust edition 2021 for all crates.

## Linting

- Workspace-level `clippy` configuration in the root `Cargo.toml`.
- Deny warnings in CI: `cargo clippy --workspace -- -D warnings`.
- Each crate may add additional `#![warn(...)]` attributes as needed.
- `rustfmt.toml` at workspace root for consistent formatting.
