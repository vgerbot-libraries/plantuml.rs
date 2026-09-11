---
title: Architecture
description: Pipeline overview, crate structure, and design patterns
---

# Architecture

plantuml.rs preserves the architecture of the Java original, mapping Java packages to Rust crates in the workspace.

## Pipeline

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

## Java Package → Rust Crate

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

## Design Patterns

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
│   ├── plantuml-java/         # Java binding (JNI)
│   └── plantuml-ts/           # TypeScript binding (WASM)
└── Cargo.toml                # Workspace manifest
```
