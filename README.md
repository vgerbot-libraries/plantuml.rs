# plantuml.rs

A 100% Rust reimplementation of [PlantUML](https://plantuml.com/) — the Java diagram-generation library — with behavioral parity as the goal, plus language bindings for Java, TypeScript, and Python via FFI.

## Status

**In progress.** Sequence diagram parsing, SVG rendering, and PREPROC (preprocessed text) output are working and tested against the Java reference. Other diagram types (class, activity, use case, etc.) and rendering backends (PNG, PDF, LaTeX) are being ported incrementally. See [`.agents/architecture.md`](.agents/architecture.md) for the full roadmap.

## Features

**Working now:**

- Sequence diagram parsing (`plantuml-sequence`)
- SVG rendering backend (`plantuml-svg`)
- PREPROC output (preprocessed text via the TIM engine, `plantuml-preproc`)
- Unified render API: `render_svg`, `render_preproc`, `render(source, format)` (`plantuml-engine`)
- Java binding via JNI — `PlantUml.renderSvg` / `PlantUml.renderPreproc`
- TypeScript binding via WASM — `renderSvg` / `renderPreproc` (browser + Node.js)
- 1D constraint solver for layout positioning (`plantuml-real`)
- Custom regex engine (`plantuml-regex`)
- Preprocessor: `!include`, `!define`, variables, conditionals (`plantuml-preproc`)
- Command parsing framework (`plantuml-command`)
- 2D graphics primitives (`plantuml-klimt`)
- Skin / style / theme system (`plantuml-skin`)

**Planned:**

- Class, activity, use case, and other diagram types
- PNG, PDF, LaTeX/TikZ rendering backends
- Layout engines (Graphviz / ELK)
- CLI binary (`plantuml-cli`)
- Python binding (PyO3 + maturin)
- C FFI shared library for additional language bindings

## Architecture

### Pipeline

```
Source text
  │
  ▼
BlockUmlBuilder          Splits input into @start/@end blocks
  │
  ▼
BlockUml                 One per @start/@end block. Lazy: TimLoader
  │                      preprocesses (!include, !define, variables)
  ▼
PSystemBuilder           createPSystem() dispatches to the
  │                      diagram-type factory (Sequence, Class,
  │                      Activity, UseCase, etc.)
  ▼
Diagram object           The in-memory diagram model
  │
  ▼
Diagram.exportDiagram()  Renders via FileFormat-specific StringBinder
  │                      to SVG / PNG / PDF / LaTeX / EPS / etc.
  ▼
Output bytes
```

### Java Package → Rust Crate

| Java Package(s) | Rust Crate | Responsibility |
|------------------|------------|---------------|
| `klimt` | `plantuml-klimt` | 2D graphics: shapes, geometry, fonts, `StringBounder`, `UGraphic`, `TextBlock` |
| `com.plantuml.ubrex` | `plantuml-regex` | Custom regex engine |
| `preproc` / `tim` | `plantuml-preproc` | Preprocessor: `!include`, `!define`, variables, conditionals |
| `command` | `plantuml-command` | `Command` / `CommandFactory` parsing framework |
| `abel` / `cucadiagram` | `plantuml-model` | Entity/relationship model: `Entity`, `Link`, `LeafType` |
| `sequencediagram` | `plantuml-sequence` | Sequence diagram |
| `skin` / `style` / `theme` | `plantuml-skin` | Skin / style / theme system |
| `real` | `plantuml-real` | 1D constraint solver for layout positioning |
| `svg` | `plantuml-svg` | SVG rendering backend |
| root (partial) | `plantuml-engine` | `BlockUml`, `BlockUmlBuilder`, `PSystemBuilder`, `SourceStringReader`, unified render API |
| root (partial) | `plantuml-core` | `Diagram`, `TextBlock`, `StringBounder`, `FileFormat`, `FileFormatOption` |
| — | `plantuml-ffi` | C FFI shared library for language bindings |
| — | `plantuml-wasm` | WASM module (wasm-bindgen) for TypeScript binding |

### Design Patterns

The Rust port preserves the key design patterns from the Java original:

1. **Factory + Registry** — `PSystemBuilder` dispatches to per-type `*DiagramFactory` implementations via a trait-based registry.
2. **Command pattern** — Each source-line parser is a `Command` registered in a `CommandFactory`; mapped to a `Command` trait with implementations.
3. **Strategy** — `FileFormat` selects the rendering backend; layout backend selection (Graphviz vs ELK) is also a strategy, using trait objects or enums.
4. **Lazy initialization** — `BlockUml.getDiagram()` builds the model on demand via `OnceCell<T>` or explicit `build()` calls.
5. **Template method** — `TitledDiagram` / `Diagram` define base behavior extended by subclasses; mapped to traits with default methods and composition.

## Workspace Structure

```
plantuml.rs/
├── crates/
│   ├── plantuml-core/         # Diagram, TextBlock, StringBounder, FileFormat
│   ├── plantuml-klimt/        # 2D graphics: shapes, geometry, fonts
│   ├── plantuml-regex/        # Custom regex engine
│   ├── plantuml-preproc/      # Preprocessor: !include, !define, variables
│   ├── plantuml-command/      # Command / CommandFactory parsing framework
│   ├── plantuml-model/        # Entity/relationship model
│   ├── plantuml-engine/       # Top-level pipeline and unified render API
│   ├── plantuml-svg/          # SVG rendering backend
│   ├── plantuml-skin/         # Skin / style / theme system
│   ├── plantuml-real/         # 1D constraint solver for layout
│   ├── plantuml-sequence/     # Sequence diagram
│   ├── plantuml-ffi/          # C FFI shared library
│   └── plantuml-wasm/         # WASM module (wasm-bindgen)
├── bindings/
│   ├── plantuml-java/         # Java binding (JNI) — com.vgerbot.plantuml:plantuml-java
│   └── plantuml-ts/           # TypeScript binding (WASM) — @vgerbot/plantuml
├── .agents/                  # Agent guidelines and architecture docs
└── Cargo.toml                # Workspace manifest
```

## Getting Started

### Prerequisites

- **Rust toolchain** (edition 2021, `resolver = "2"`)
- **Java 17+** — required only for the Java binding
- **Node.js** — required only for the TypeScript binding (`npm install` in `bindings/plantuml-ts`)

### Build

```sh
cargo build
```

### Test

```sh
cargo test
```

### Lint

```sh
cargo clippy --workspace -- -D warnings
```

## Usage

### Rust

```rust
use plantuml_engine::render_svg;

let svg = render_svg("@startuml\nAlice -> Bob: hello\n@enduml")?;
```

The `render` function dispatches by `FileFormat`:

```rust
use plantuml_engine::render;
use plantuml_core::FileFormat;

let svg = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Svg)?;
```

### Java

Maven coordinates:

```xml
<dependency>
    <groupId>com.vgerbot.plantuml</groupId>
    <artifactId>plantuml-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

Usage:

```java
import com.vgerbot.plantuml.PlantUml;

String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\n@enduml");
System.out.println(svg);
```

### TypeScript

Install:

```sh
npm install @vgerbot/plantuml
```

Usage (browser or Node.js):

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
```

### Python

Planned (PyO3 + maturin). Not yet implemented.

## Binding Builds

| Binding | Build Command |
|---------|-------------|
| Java | `cd bindings/plantuml-java && ./gradlew build` |
| TypeScript | `cd bindings/plantuml-ts && npm run build` |

## Testing

```sh
cargo test --workspace
```

Tests are ported directly from the PlantUML Java reference:

- **Vega data-driven tests** — `.puml` files with expected `.svg` / `.preproc` output, compared against Java reference data. Covers 38 sequence SVG tests and PREPROC tests across the `asciiverse/`, `svg/`, and `mvp/` suites.
- **Nonreg tests** — regression tests ported from the Java nonreg suite.
- **Unit / misc tests** — per-crate unit tests in `#[cfg(test)] mod tests`.

Test data is committed under `crates/plantuml-engine/tests/resources/vega/`.

## Porting Guidelines

The port follows a strict 1:1 Java → Rust file mapping:

- **Naming**: PascalCase → snake_case for files and methods; Java package → Rust crate path.
- **File mapping**: one Java file → one Rust file; file size governance (≤500 / 501–800 / >800 lines).
- **OOP → Rust**: interfaces → traits, classes → structs, overloads → builders.
- **Error handling**: `Result` + `thiserror` for libraries, `anyhow` for CLI. No panics in library code.
- **Doc comments**: all public items cite the Java source, e.g. `/// Ported from: net/sourceforge/plantuml/SourceStringReader.java`.

See [`.agents/rules/java-to-rust-porting.md`](.agents/rules/java-to-rust-porting.md) for full details.

## Contributing

See [`.agents/AGENTS.md`](.agents/AGENTS.md) for project conventions and [`.agents/architecture.md`](.agents/architecture.md) for the architecture overview, layer mapping, and design patterns.

## License

MIT License (per the workspace [`LICENSE`](LICENSE) file).

> **Note:** `Cargo.toml` declares `license = "GPL-3.0-only"` in the workspace package metadata, but the authoritative `LICENSE` file is MIT. This README reflects the `LICENSE` file. If the project intends GPL-3.0, the `LICENSE` file and `Cargo.toml` metadata need reconciliation.
