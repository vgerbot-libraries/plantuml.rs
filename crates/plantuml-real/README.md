# plantuml-real

1D constraint solver for layout positioning.

Ported from: `net/sourceforge/plantuml/real/` package (13 files).

## Overview

The Real system models positions as constraint variables connected by positive forces. `RealLine::compile()` iterates all forces until a fixed point is reached, ensuring all minimum-distance constraints are satisfied. This is used by diagram layout engines to compute x and y coordinates of elements.

## Key Types

| Type | Description |
|------|-------------|
| `RealLine` | Constraint solver holding all forces; iterates to fixed point |
| `Real` (trait) | 1D position variable in the constraint system |
| `RealImpl` | Mutable real value — the only type directly moved by forces |
| `RealDelta` | Real at `delegated + diff` (fixed offset) |
| `RealDeltaLive` | Real at `delegated + delta()` where `delta()` is a live closure |
| `RealMin` | Minimum of multiple reals |
| `RealMax` | Maximum of multiple reals |
| `RealMiddle2` | Midpoint of two movable reals |
| `PositiveForce` | Constraint ensuring `moving_point >= fixed_point + min_distance` |

## Free Functions

- `create_origin()` — new origin real at position 0
- `add_fixed(real, delta)` — real at `real + delta` (no constraint)
- `add_at_least(real, delta)` — mutable real with force ensuring `>= delta` ahead
- `ensure_bigger_than(real, other)` — constraint `real >= other`
- `middle(r1, r2)` — midpoint of two reals
- `max(reals)` / `min(reals)` — max/min of multiple reals
- `with_live_offset(real, delta)` — real with live closure offset
- `compile_now(line)` — compile all constraints on a line
- `clear_forces(line)` — clear all forces (breaks reference cycles)

## Usage

```rust
use plantuml_real::{create_origin, add_at_least, compile_now};
use std::cell::RefCell;
use std::rc::Rc;

let line = Rc::new(RefCell::new(plantuml_real::RealLine::default()));
let origin = create_origin();
let point = add_at_least(&origin, 100.0);
// ... add more constraints ...
compile_now(&line);
let pos = point.get_currentValue();
```

## License

MIT License (per workspace `LICENSE` file).
