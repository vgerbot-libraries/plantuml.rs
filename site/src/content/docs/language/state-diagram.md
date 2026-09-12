---
title: State Diagram
description: State diagram syntax and examples in plantuml.rs
---

# State Diagram

State diagrams describe the states of an object and the transitions between them — states, transitions, guards, and actions.

## Declaring states

Declare states with the `state` keyword. Use `[*]` to denote the initial and final pseudo-states.

```plantuml
@startuml
state Idle
state Active
[*] -> Idle
Idle --> Active : start
Active --> [*] : stop
@enduml
```

## Transitions with labels

Add labels to transitions to describe the event or condition that triggers them.

```plantuml
@startuml
state Off
state On
state Dim
[*] --> Off
Off --> On : switch
On --> Dim : dimmer
Dim --> On : brighter
On --> Off : switch
@enduml
```

## Composite states

Nest states inside a composite state using braces.

```plantuml
@startuml
state Active {
  state Running
  state Paused
  Running --> Paused : pause
  Paused --> Running : resume
}
[*] --> Active
Active --> [*] : done
@enduml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
