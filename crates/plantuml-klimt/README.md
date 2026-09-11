# plantuml-klimt

2D graphics abstraction layer for plantuml.rs.

Ported from: `net/sourceforge/plantuml/klimt/` (drawing, shape, font, color).

## Overview

Provides the graphics primitives and abstractions used by all rendering backends. The crate ports the deterministic text-measurement path (width table + `StringBounderFromWidthTable`) and the `UGraphic`/`UShape`/`UChange`/`UDriver` trait surface. Shapes, colors, and backend-specific drivers are added as their backends are ported.

## Modules

| Module | Description |
|--------|-------------|
| `ugraphic` | `UGraphic` trait — the graphics context abstraction |
| `udriver` | `UDriver` trait — backend-specific shape rendering |
| `ushape` | `UShape` trait — drawable shape abstraction |
| `uchange` | `UChange` trait — context change (color, font, stroke, translate, clip) |
| `changes` | Concrete changes: `UChangeColor`, `UChangeFont`, `UChangeStroke`, `UTranslate`, `UClip` |
| `color` | Color types: `HColor`, `HColorSet`, `HColorSimple`, `ColorMapper` |
| `shapes` | Shape structs: `UEllipse`, `ULine`, `UPath`, `UPolygon`, `URectangle`, `UText` |
| `string_bounder_from_width_table` | `StringBounderFromWidthTable` — deterministic text measurement |
| `unicode_block` | Unicode block classification |
| `unicode_font_width_sans_serif` | Built-in width table for sans-serif fonts |

## Key Exports

- `UGraphic` — graphics context that dispatches shape draws to a `UDriver`
- `UShape` — trait for ellipses, lines, paths, polygons, rectangles, text
- `HColor` / `HColorSet` — color representation and palette
- `StringBounderFromWidthTable` — deterministic text width measurement

## License

MIT License (per workspace `LICENSE` file).
