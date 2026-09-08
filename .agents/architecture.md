# Architecture

## Project Goal

plantuml.rs is a 100% Rust reimplementation of [PlantUML](https://plantuml.com/), the Java diagram-generation library. The goal is behavioral parity with the Java original, plus language bindings for Java, TypeScript, and Python via FFI.

## PlantUML Pipeline

The PlantUML processing pipeline, derived from analysis of the reference Java source:

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

### Key entry points (Java → Rust crate)

| Java class | Role | Rust crate |
|------------|------|------------|
| `SourceStringReader` | Programmatic API entry point | `plantuml-engine` |
| `BlockUmlBuilder` | Splits source into `@start`/`@end` blocks | `plantuml-engine` |
| `BlockUml` | One block; lazy preprocessing via `TimLoader` | `plantuml-engine` |
| `PSystemBuilder` | Diagram-type dispatcher (factory + registry) | `plantuml-engine` |
| `FileFormat` | Output format enum | `plantuml-core` |

## Layer Mapping (Java Package → Rust Crate)

| Java Package(s) | Rust Crate | Responsibility |
|-----------------|------------|---------------|
| `klimt` | `plantuml-klimt` | 2D graphics: shapes, geometry, fonts, `StringBounder`, `UGraphic`, `TextBlock` |
| `com.plantuml.ubrex` | `plantuml-regex` | Custom regex engine |
| `preproc` / `tim` | `plantuml-preproc` | Preprocessor: `!include`, `!define`, variables, conditionals |
| `command` | `plantuml-command` | `Command` / `CommandFactory` parsing framework |
| `abel` / `cucadiagram` | `plantuml-model` | Entity/relationship model: `Entity`, `Link`, `LeafType` |
| `sequencediagram` | `plantuml-sequence` | Sequence diagram |
| `classdiagram` | `plantuml-class` | Class diagram |
| `activitydiagram3` | `plantuml-activity` | Activity diagram |
| `skin` / `style` / `theme` | `plantuml-skin` | Skin / style / theme system |
| `svek` / `elk` / `sdot` | `plantuml-layout` | Layout engines (Graphviz / ELK) |
| `svg` | `plantuml-svg` | SVG rendering backend |
| `png` | `plantuml-png` | PNG rendering backend |
| `openpdf` | `plantuml-pdf` | PDF rendering backend |
| `tikz` | `plantuml-pdf` | LaTeX/TikZ rendering (combined with PDF) |
| root (partial) | `plantuml-engine` | `BlockUml`, `BlockUmlBuilder`, `PSystemBuilder`, `SourceStringReader` |
| root (partial) | `plantuml-core` | `Diagram`, `TextBlock`, `StringBounder`, `FileFormat`, `FileFormatOption` |
| `Run` / `cli` | `plantuml-cli` | CLI binary |
| — | `plantuml-ffi` | C FFI shared library for language bindings |
| — | `bindings/plantuml-java` | Java bindings (JNI) |
| — | `bindings/plantuml-python` | Python bindings (PyO3 + maturin) |
| — | `bindings/plantuml-ts` | TypeScript bindings (napi-rs) |

## Key Design Patterns to Preserve

1. **Factory + Registry**: `PSystemBuilder` dispatches to per-type `*DiagramFactory` implementations. Each diagram type registers its factory. The Rust port should use a trait-based registry pattern.

2. **Command pattern**: Each source-line parser is a `Command` registered in a `CommandFactory`. The Rust port maps this to a `Command` trait with implementations registered in a `CommandFactory` struct.

3. **Strategy**: `FileFormat` selects the rendering `StringBinder`. Layout backend selection (Graphviz vs ELK) is also a strategy. Rust uses trait objects (`Box<dyn ...>`) or enums for strategy selection.

4. **Lazy initialization**: `BlockUml.getDiagram()` builds the model on demand, not at construction. Rust preserves this with lazy evaluation (e.g., `OnceCell<T>` or explicit `build()` calls).

5. **Template method**: `TitledDiagram` / `Diagram` define base behavior that subclasses extend. Rust uses traits with default methods and composition rather than inheritance.

## Rendering Backends

| Backend | Java Technology | Rust Replacement |
|---------|----------------|------------------|
| SVG | Direct XML generation | Direct XML generation (primary, same approach) |
| PNG | Java AWT / Java2D | `tiny-skia` or `resvg` crate |
| PDF | OpenPDF | `pdf-writer` or `printpdf` crate |
| LaTeX/TikZ | Direct string generation | Direct string generation |
| Braille / ASCII | Direct string generation | Direct string generation |

SVG is the primary backend and should be implemented first, as it requires no external rendering dependency.

## Binding Strategy

All bindings expose the same core API: given PlantUML source text and an output format, produce rendered output bytes.

### Java (JNI)

- `plantuml-ffi` exposes a C ABI (via `cbindgen`).
- `bindings/plantuml-java` provides a Java JAR that loads the shared library via JNI and wraps the C ABI calls.
- This allows drop-in replacement for users of the Java PlantUML API.

### Python (PyO3)

- `bindings/plantuml-python` uses PyO3 to create a native Python extension module.
- Built with `maturin develop` / `maturin build`.
- Direct Rust ↔ Python interop; no C ABI layer needed.

### TypeScript (napi-rs)

- `bindings/plantuml-ts` uses napi-rs to create a native Node.js addon.
- Built with `npm run build`.
- Direct Rust ↔ Node.js interop; no C ABI layer needed.

## Scale

- ~1500–1900 Java source files across 112 sub-packages.
- ~250k–350k lines of Java code.
- Porting will be incremental, starting with foundational layers and building up:
  1. `plantuml-klimt` (2D graphics primitives)
  2. `plantuml-core` (shared traits and types)
  3. `plantuml-command` (parsing framework)
  4. `plantuml-sequence` (first diagram type)
  5. `plantuml-svg` (first rendering backend)
  6. Remaining diagram types and backends
  7. `plantuml-cli` (CLI)
  8. FFI and language bindings

## Dependencies (Java → Rust Replacement)

| Java Dependency | Purpose | Rust Replacement |
|-----------------|---------|------------------|
| Eclipse ELK | Layout engine | `layout` crate or port ELK algorithms |
| OpenPDF | PDF generation | `pdf-writer` or `printpdf` crate |
| JLaTeXMath | Math rendering | Math rendering crate or port |
| Java AWT / Java2D | 2D graphics for PNG | `tiny-skia` or `resvg` crate |
| Graphviz (C-to-Java port) | Graph layout | Evaluate `layout` crate or re-port from C |
