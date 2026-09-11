# plantuml-svg

SVG rendering backend for plantuml.rs.

Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/` package.

## Overview

Implements the SVG output backend. `UGraphicSvg` implements the `UGraphic` trait from `plantuml-klimt`, dispatching shape draws to `SvgGraphics` which generates SVG XML elements. The XML document model (`XmlDocument`, `XmlNode`, `XmlWriter`) provides a lightweight XML builder for SVG output.

## Modules

| Module | Description |
|--------|-------------|
| `ugraphic_svg` | `UGraphicSvg` — `UGraphic` implementation for SVG output; `SvgChange` — change tracking |
| `svg_graphics` | `SvgGraphics` — low-level SVG element generation; `TransparentFillBehavior` |
| `svg_option` | `SvgOption` / `LengthAdjust` — SVG output options |
| `xml` | XML document model: `XmlDocument`, `XmlNode`, `XmlLeaf`, `XmlWriter` |

## Key Exports

- `UGraphicSvg` — graphics context that renders to SVG
- `SvgGraphics` — SVG element and attribute generation
- `XmlDocument` / `XmlWriter` — XML document construction and serialization
- `SvgOption` — SVG rendering options

## License

MIT License (per workspace `LICENSE` file).
