# plantuml-skin

PlantUML skin, style, and theme system.

Ported from: `net/sourceforge/plantuml/skin/` and `net/sourceforge/plantuml/style/` packages.

## Overview

Controls the visual appearance of diagrams. `SkinParam` holds all rendering parameters (colors, fonts, paddings, arrow styles). The style system (`Style`, `StyleBuilder`, `StyleQuery`) provides a CSS-like cascade for per-element styling. Themes bundle sets of skin parameters and styles.

## Modules

| Module | Description |
|--------|-------------|
| `skin_param` | `SkinParam` — the main skin parameter container |
| `style` | `Style` — individual style with specificity |
| `style_builder` | `StyleBuilder` / `AutomaticCounter` — style construction |
| `style_query` | `StyleQuery` / `StyleAtom` — style lookup query |
| `p_name` | `PName` — parameter name enum (all skin parameter names) |
| `s_name` | `SName` — style name enum |
| `pragma` | `Pragma` / `Warning` — pragma directives |
| `pragma_key` | `PragmaKey` — pragma key type |
| `specificity` | `Specificity` — CSS-like specificity for style cascade |
| `component` | `Area`, `ComponentType`, `Context2D`, `SimpleContext2D` — component layout |
| `arrow` | Arrow types: `ArrowConfiguration`, `ArrowHead`, `ArrowBody`, `ArrowDressing` |
| `placeholder_types` | Parameter value types: `AlignmentParam`, `ColorParam`, `FontParam`, `Rankdir`, etc. |
| `clockwise_top_right_bottom_left` | `ClockwiseTopRightBottomLeft` — margin/padding descriptor |
| `is_skin_param` | `ISkinParam` trait, `SWIMLANE_WIDTH_SAME` constant |
| `length_adjust` | `LengthAdjust` — text length adjustment strategy |
| `merge_strategy` | `MergeStrategy` — style merge strategy |
| `value` | `Value`, `DarkString`, `HorizontalAlignment`, `UFontFace` — style value types |

## Key Exports

- `SkinParam` — main skin parameter container
- `Style` / `StyleBuilder` — style definition and construction
- `StyleQuery` — style cascade lookup
- `PName` / `SName` — parameter and style name enums
- `Pragma` / `PragmaKey` — pragma directives
- `Specificity` — style priority for cascade resolution

## License

MIT License (per workspace `LICENSE` file).
