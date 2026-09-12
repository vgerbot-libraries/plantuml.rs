---
title: Timing Diagram
description: Timing diagram syntax and examples in plantuml.rs
---

# Timing Diagram

Timing diagrams describe changes in state over time — digital signals, clocks, and binary lifelines. plantuml.rs supports the `@starttiming` / `@endtiming` block syntax.

## Binary signals

Declare a binary signal with `binary` and set its value with `@time` markers followed by `is low` / `is high` (or `= 0` / `= 1`).

```plantuml
@starttiming
binary "Signal A" as A
@0
A is low
@5
A is high
@10
A is low
@endtiming
```

## Clock signal

Declare a clock with `clock` and a period. The waveform repeats automatically.

```plantuml
@starttiming
clock "clk" as C with period 10
@endtiming
```

## Multiple signals

Combine several signals in one diagram. Each signal gets its own lane stacked vertically.

```plantuml
@starttiming
clock "clk" as C with period 10
binary "Data" as D
@0
D is low
@5
D is high
@15
D is low
@endtiming
```

---

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
