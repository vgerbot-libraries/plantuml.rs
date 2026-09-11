# plantuml-core

Foundational types shared by all plantuml crates.

Ported from the `net.sourceforge.plantuml` root and `core` packages.

## Modules

| Module | Description |
|--------|-------------|
| `diagram` | `Diagram` trait — the core diagram abstraction |
| `diagram_description` | `DiagramDescription` metadata |
| `diagram_type` | `DiagramType` enum (sequence, class, activity, etc.) |
| `p_system_error` | `PSystemError` / `ErrorUmlType` — factory error types |
| `file_format` | `FileFormat` enum (SVG, PNG, PDF, PREPROC, etc.) |
| `file_format_option` | `FileFormatOption` wrapper |
| `geom` | Geometry types: `XPoint2D`, `XDimension2D`, `XLine2D`, `XRectangle2D` |
| `string_bounder` | `StringBounder` trait for text measurement |
| `text_block` | `TextBlock` trait for renderable text blocks |
| `u_font` | `UFont` font descriptor |

## Key Exports

- `PSystemError` / `ErrorUmlType` — error from diagram-type factories
- `DiagramType` — diagram type enum with `find_start_types()` dispatch
- `FileFormat` / `FileFormatOption` — output format selection
- `TextBlock` — renderable text block abstraction
- `StringBounder` — text width measurement interface
- `UFont` — font family, size, style descriptor
- Geometry primitives: `XPoint2D`, `XDimension2D`, `XLine2D`, `XRectangle2D`

## Usage

```rust
use plantuml_core::{FileFormat, FileFormatOption};

let opt = FileFormatOption::new(FileFormat::Svg);
```

## License

MIT License (per workspace `LICENSE` file).
