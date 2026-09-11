# plantuml-preproc

PlantUML preprocessor: `!include`, `!define`, variables, conditionals, and the TIM engine.

Ported from `net.sourceforge.plantuml.tim`, `net.sourceforge.plantuml.preproc`, and `net.sourceforge.plantuml.preproc2` packages.

## Overview

Handles all preprocessing of PlantUML source text before diagram parsing. The TIM engine (`TimLoader`) processes `!include` directives, `!define` macros, variable substitution, conditional blocks (`!if`/`!else`/`!endif`), loops (`!while`/`!endwhile`), and function definitions. The preprocessor output is the expanded source text consumed by the diagram-type factories.

## Modules

| Module | Description |
|--------|-------------|
| `tim` | TIM engine: `TimLoader`, `TContext`, `TMemory`, `TFunction`, `Knowledge`, `Token`, `Trie` |
| `preproc` | Preprocessor pipeline (v1) |
| `preproc2` | Preprocessor pipeline (v2) |
| `stubs` | Stub implementations for unported preproc dependencies |
| `string_located` | `StringLocated` — source text with position tracking |
| `t_line_type` | `TLineType` — line classification enum |

## Key Exports

- `TimLoader` — main preprocessor entry point
- `TContext` / `TMemory` — execution context and memory state
- `TFunction` / `TFunctionImpl` — function definition and implementation
- `Knowledge` — variable and function registry
- `Token` / `TokenType` / `TokenStack` — token types for the TIM lexer
- `StringLocated` — source line with position metadata

## License

MIT License (per workspace `LICENSE` file).
