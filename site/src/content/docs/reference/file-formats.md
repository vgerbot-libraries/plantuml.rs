---
title: File Formats
description: FileFormat enum variants and supported formats
---

# File Formats

The `FileFormat` enum (from `plantuml-core`, ported from `net.sourceforge.plantuml.FileFormat.java`) selects the rendering backend.

## Variants

```rust
pub enum FileFormat {
    Eps,
    EpsText,
    Atxt,
    Utxt,
    XmiStandard,
    XmiStar,
    XmiArgo,
    XmiCustom,
    XmiScript,
    Scxml,
    Graphml,
    Pdf,
    Html,
    Html5,
    Vdx,
    Latex,
    LatexNoPreamble,
    LatexDeterministic,
    Base64,
    BraillePng,
    Obfuscate,
    Debug,
    Null,
    Preproc,
    Png,
    PngEmpty,
    Raw,
    SvgDeterministic,
    Svg,
}
```

## Supported

| Variant | Status |
|---|---|
| `Svg` | ✅ Working — sequence diagrams |
| `Preproc` | ✅ Working — preprocessed text output |
| `Png` | 🚧 Planned |
| `Pdf` | 🚧 Planned |
| `Latex` | 🚧 Planned |
| All others | 🚧 Planned |

## Usage

```rust
use plantuml_core::FileFormat;
use plantuml_engine::render;

let svg = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Svg)?;
```

Unsupported formats return `RenderError::UnsupportedFormat(format)`.
