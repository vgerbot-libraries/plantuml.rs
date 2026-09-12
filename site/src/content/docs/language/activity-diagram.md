---
title: Activity Diagram
description: Activity diagram syntax and examples in plantuml.rs
---

# Activity Diagram

Activity diagrams describe workflows and business processes — sequential and parallel activities, decisions, and flows. plantuml.rs supports the `activitydiagram3` syntax (the beta/new activity diagram).

## Basic flow

Use `start` and `stop` to mark the beginning and end of the flow. Each activity is written as `:label;`.

```plantuml
@startuml
start
:Do something;
:Do another thing;
stop
@enduml
```

## If / else

Use `if (condition) then (label)` … `else (label)` … `endif` for branching.

```plantuml
@startuml
start
if (condition?) then (yes)
  :Take yes path;
else (no)
  :Take no path;
endif
stop
@enduml
```

## While loop

Use `while (condition) is (label)` … `endwhile` for iteration.

```plantuml
@startuml
start
while (more data?) is (yes)
  :Process item;
endwhile (no)
stop
@enduml
```

---

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
