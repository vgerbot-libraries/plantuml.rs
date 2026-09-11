# plantuml-regex

Custom regex engine for plantuml.rs.

Ported from: `net/sourceforge/plantuml/ubrex/` (Unicode bracket notation regex).

## Overview

PlantUML uses its own regex dialect called **ubrex** (Unicode Bracket Regex) that supports Java's `Pattern`-like syntax with Unicode bracket expressions. This crate provides a from-scratch Rust implementation of that engine, plus a legacy wrapper that provides a `regex`-compatible API for code paths that still use standard regex patterns.

## Modules

| Module | Description |
|--------|-------------|
| `ubrex` | Unicode bracket notation regex engine (ported from `com.plantuml.ubrex`) |
| `legacy` | Wrapper providing a standard regex API for compatibility |

## Usage

```rust
use plantuml_regex::ubrex;
```

## License

MIT License (per workspace `LICENSE` file).
