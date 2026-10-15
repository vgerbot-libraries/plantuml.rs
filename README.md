# plantuml.rs

A Rust reimplementation of [PlantUML](https://plantuml.com/), the Java
diagram-generation library. The goal is behavioral parity with the Java
original (output compared against the Java reference), plus language
bindings on top of a single Rust engine.

## Status

Active development. SVG is the only implemented graphics backend; it works
for all diagram types listed below and is continuously checked against Java
PlantUML for byte-level parity after SVG normalization.

**Implemented**

- Fourteen diagram kinds rendered to SVG:
  sequence, class, object, state, component, deployment, use case, activity,
  timing, Gantt, mindmap, WBS, JSON, and YAML.
- PREPROC output (preprocessed source text) via the TIM preprocessor:
  `!include` / `!includesub`, `!define`, variables, conditionals,
  user procedures and functions, math/boolean evaluation, stdlib and theme
  loading, and the JSON built-in functions.
- Graph layout for node/edge diagrams: an external Graphviz `dot` is used
  when it is on `PATH`, with a pure-Rust port of the relevant `dot` subset
  as the fallback (always used in the WASM build; forceable anywhere with
  `PLANTUML_NO_DOT`).
- Unified Rust render API (`render_svg`, `render_preproc`, `render`).
- Three binding layers over the same API: a C ABI shared library
  (`plantuml-ffi`), a JNI library plus Java class (`bindings/plantuml-java`),
  and a WebAssembly module with a TypeScript package for browser and Node.js
  (`plantuml-wasm`, `bindings/plantuml-ts`).
- Supporting subsystems: 2D graphics primitives, the UBrex regex engine,
  the command parsing framework, skin/style/theme handling, and a 1D
  constraint solver.
- An Astro/Starlight documentation site with an in-browser WASM playground
  (`site/`).

**Not yet implemented**

- Output formats other than SVG and PREPROC (PNG, PDF, EPS, LaTeX/TikZ,
  ASCII, XMI, SCXML, GraphML, …). The render API returns
  `UnsupportedFormat` for them.
- Diagram types not listed above (ditaa, salt, BPMN, wire, board, git, …).
- A standalone CLI binary.
- Python bindings (planned: PyO3 + maturin).
- Crate/package publication: no Rust, Java, or TypeScript package is
  currently published; the workspace declares `publish = false` in its
  package metadata, and neither the Java nor the TypeScript package is on
  Maven Central or npmjs.org.

## Supported diagrams

| `@start…` keyword | Diagram | Implementation |
|---|---|---|
| `@startuml` (messages) | Sequence | `plantuml-sequence` |
| `@startuml` (`class`, `object`) | Class / Object | `plantuml-cuca`, `plantuml-model` |
| `@startuml` (`state`) | State | `plantuml-cuca`, `plantuml-model` |
| `@startuml` (`component`, `node`, `usecase`, `actor`, …) | Component / Deployment / Use case | `plantuml-cuca`, `plantuml-model` |
| `@startuml` (`start`/`stop` legacy syntax) | Activity | `plantuml-activity` |
| `@starttiming` | Timing | `plantuml-timing` |
| `@startgantt` | Gantt | `plantuml-gantt` |
| `@startmindmap` | Mindmap | `plantuml-mindmap` |
| `@startwbs` | Work breakdown structure | `plantuml-mindmap` |
| `@startjson` | JSON data | `plantuml-json` |
| `@startyaml` | YAML data (converted to JSON) | `plantuml-json` |

Class, object, state, component, deployment, and use case diagrams share the
CucaDiagram infrastructure in `plantuml-cuca` (entity/link model plus
graph layout and edge routing).

## Architecture

### Pipeline

```
Source text
  │
  ▼
BlockUmlBuilder          Splits input into @start/@end blocks and runs
  │                      the TIM preprocessor (!include, !define, …)
  ▼
BlockUml / UmlSource     One preprocessed block plus its candidate
  │                      diagram types
  ▼
PSystemBuilder          Tries the registered diagram factories in order;
  │                      first match wins (sequence, class, object, state,
  │                      description, timing, gantt, activity, json, yaml,
  │                      mindmap, wbs)
  ▼
Diagram                 The in-memory diagram model (Diagram trait)
  ▼
Diagram::export_diagram Renders through the selected backend
  │
  ▼
SVG / PREPROC bytes
```

### Crates

| Crate | Responsibility |
|---|---|
| `plantuml-core` | Core traits and types: `Diagram`, `FileFormat`, `FileFormatOption`, `DiagramType`, `PSystemError` |
| `plantuml-klimt` | 2D graphics: shapes, geometry, fonts, `StringBounder`, `UGraphic`, `TextBlock` (port of the Java `klimt` package) |
| `plantuml-regex` | Custom UBrex regex engine |
| `plantuml-preproc` | TIM preprocessor and directives |
| `plantuml-command` | `Command` / `CommandFactory` source-line parsing framework |
| `plantuml-model` | Entity/relationship model: `Entity`, `Link`, `LeafType`, … |
| `plantuml-real` | 1D constraint solver for layout positioning |
| `plantuml-skin` | Skin parameters, styles, and themes |
| `plantuml-svg` | SVG rendering backend (direct XML generation) |
| `plantuml-sequence` | Sequence diagram |
| `plantuml-cuca` | Shared CucaDiagram engine for class/object/state/description diagrams, including pure-Rust `dot`-compatible layout and spline routing |
| `plantuml-activity` | Activity diagram |
| `plantuml-timing` | Timing diagram |
| `plantuml-gantt` | Gantt diagram |
| `plantuml-mindmap` | Mindmap and WBS diagrams |
| `plantuml-json` | JSON and YAML diagrams (`serde_json` / `serde_yaml`) |
| `plantuml-engine` | Top-level pipeline (`BlockUmlBuilder`, `BlockUml`, `PSystemBuilder`, `SourceStringReader`) and the unified render API |
| `plantuml-ffi` | C ABI shared library (`cdylib`), header generated with cbindgen |
| `plantuml-wasm` | WebAssembly bindings via `wasm-bindgen` (`cdylib`) |
| `bindings/plantuml-java/native` (`plantuml-jni`) | JNI native library used by the Java package |

The design follows the Java original: factory + registry dispatch in
`PSystemBuilder`, one `Command` per source-line parser, strategy-style
backend selection through `FileFormat`, and composition instead of the
`TitledDiagram` inheritance hierarchy.

### Layout

When a Graphviz `dot` executable is on `PATH`, the engine sends the graph to
`dot -Tsvg` and parses the result back; this mirrors the reference Smetana
output for the covered diagrams. When `dot` is unavailable
(including the WASM build), a built-in pure-Rust solver runs instead — a
port of the relevant subset of Graphviz 14 (`dotgen` ranking and node
placement, edge-label rank doubling, and `pathplan` spline routing).
Setting `PLANTUML_NO_DOT` to any value forces the pure-Rust path; the
Java-parity test suites always set it so an external `dot` cannot mask
layout differences.

## Repository layout

```
plantuml.rs/
├── crates/                    # Rust workspace crates (see table above)
├── bindings/
│   ├── plantuml-java/         # Java package (Gradle) + native/ JNI crate
│   └── plantuml-ts/           # @vgerbot/plantuml TypeScript package
├── site/                      # Astro + Starlight docs site and playground
├── scripts/                   # init.sh — toolchain and artifact setup
├── .github/workflows/         # CI (clippy, tests, Java parity) and site deploy
└── Cargo.toml                 # Workspace manifest
```

## Requirements

- Rust stable, edition 2021.
- For the TypeScript/WASM build: the `wasm32-unknown-unknown` target,
  [`wasm-pack`](https://rustwasm.github.io/wasm-pack/), and Node.js
  (the repository pins Node 22 in `.nvmrc`).
- For the Java binding: JDK 17 or newer and Gradle.
- For parity tests: a Java runtime and the PlantUML reference JAR.
- Graphviz `dot` is optional (see [Layout](#layout)).

`scripts/init.sh` installs the prerequisites and builds the Rust crates,
WASM modules, TypeScript package, and site in one step
(`--rust`, `--wasm`, `--ts`, `--site`, `--check` flags available).

## Build, test, lint

```sh
cargo build
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Usage

### Rust

```rust
fn main() -> Result<(), plantuml_engine::RenderError> {
    let svg = plantuml_engine::render_svg("@startuml\nAlice -> Bob: hello\n@enduml")?;
    println!("{svg}");
    Ok(())
}
```

Format-explicit dispatch:

```rust
use plantuml_core::FileFormat;
use plantuml_engine::render;

let svg = render(source, FileFormat::Svg)?;
let preproc = render(source, FileFormat::Preproc)?;
```

`render` supports `Svg` and `Preproc`; every other `FileFormat` returns
`RenderError::UnsupportedFormat`.

### C (C ABI)

`plantuml-ffi` builds a `cdylib`; `crates/plantuml-ffi/include/plantuml.h`
is generated by cbindgen:

```c
#include "plantuml.h"

char *out = NULL;
if (plantuml_render_svg(source, &out) == 0) {
    /* use out */
    plantuml_free_string(out);
}
```

The functions return `0` on success, `-1` on parse error/null input,
`-2` on invalid UTF-8 input, and `-3` if the output contains a NUL byte.

### Java

```sh
cd bindings/plantuml-java
gradle build        # runs cargo build --release in native/ and packages the .so/.dylib/.dll into the JAR
```

```java
import com.vgerbot.plantuml.PlantUml;

String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\n@enduml");
```

The native library is loaded from `java.library.path` when present and is
otherwise extracted from the JAR's `/native` resource at runtime. The
package coordinates are `com.vgerbot.plantuml:plantuml-java:0.1.0`; build it
locally, it is not published.

### TypeScript / WebAssembly

```sh
cd bindings/plantuml-ts
npm install
npm run build       # wasm-pack web + nodejs targets, then tsc
```

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
```

The package ships both WASM targets and auto-detects browser versus Node.js.
The site consumes it through a local `file:` dependency; it is not published
to npmjs.org.

## Parity testing

Output correctness is established by comparing the Rust renderer against
the Java PlantUML reference:

- `tests/java_parity_test.rs` renders 55 examples copied from the language
  documentation (every supported diagram kind) with both implementations,
  normalizes both SVGs, and asserts equality. The pure-Rust layout path is
  forced with `PLANTUML_NO_DOT=1`.
- `tests/vega_test.rs` is the port of the Java `test.vega.VegaTest`
  harness over a 205-case `.puml` corpus under
  `crates/plantuml-engine/tests/resources/vega/` (metadata in `vega.json`
  and `vega-summary.md`). Enabled cases obtain expected output from the JAR
  at test time and compare after normalization; cases not yet at parity are
  `#[ignore]`d. Only `.puml` inputs are committed — expected outputs are
  generated at runtime, never stored.

The JAR is resolved from `PLANTUML_JAR`, or from
`crates/plantuml-engine/tests/vendor/plantuml.jar` (git-ignored). CI
downloads PlantUML 1.2026.6 from Maven Central. Tests that cannot find Java
or the JAR skip gracefully, except the documentation parity suite, which
fails loudly.

## Documentation

- User documentation and the WASM playground live in `site/`
  (Astro + Starlight; deployed to GitHub Pages); language pages cover all
  supported diagram types.
- Porting and architecture documentation for contributors lives under
  [`.agents/`](.agents/): [`architecture.md`](.agents/architecture.md),
  porting rules, and test-parity rules in [`.agents/AGENTS.md`](.agents/AGENTS.md).

The port follows a strict 1:1 Java → Rust file mapping (Java packages map
to crates, PascalCase to snake_case), uses traits/composition instead of
inheritance, `Result` with `thiserror` in libraries, and cites the Java
source file in every public item's doc comment.

## License

MIT License — see [`LICENSE`](LICENSE) (© 2026 vgerbot-libraries).
