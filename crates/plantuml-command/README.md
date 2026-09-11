# plantuml-command

Command framework for PlantUML source-line parsing.

Ported from: `net/sourceforge/plantuml/command/` package.

## Overview

Implements the Command pattern used by diagram parsers. Each source line is matched against registered `Command` implementations. Commands can be single-line or multi-line, and use the ubrex regex engine for pattern matching. The framework handles command registration, matching, execution, and multi-line command continuation.

## Modules

| Module | Description |
|--------|-------------|
| `command` | `Command` trait — the core command abstraction |
| `command_control` | `CommandControl` — flow control (continue, stop, error) |
| `command_execution_result` | `CommandExecutionResult` — result of command execution |
| `command_multilines` | `CommandMultilines2` — multi-line command framework |
| `single_line_command` | `SingleLineCommand2` — single-line command implementation |
| `multilines_strategy` | `MultilinesStrategy` — strategy for multi-line command behavior |
| `ubrex_command_multilines` | `UbrexCommandMultilines2` — multi-line command with ubrex patterns |
| `ubrex_single_line_command` | `UbrexSingleLineCommand2` — single-line command with ubrex patterns |
| `parser_pass` | `ParserPass` — parsing pass coordination |
| `trim` | `Trim` — whitespace trimming utilities |

## Key Exports

- `Command` — trait for source-line parsers
- `SingleLineCommand2` / `CommandMultilines2` — concrete command implementations
- `CommandControl` — flow control enum
- `CommandExecutionResult` — execution outcome
- `MultilinesStrategy` — multi-line command behavior strategy

## License

MIT License (per workspace `LICENSE` file).
