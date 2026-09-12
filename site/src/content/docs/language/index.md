---
title: Language Reference
description: Overview of all PlantUML diagram types and their implementation status
---

# Language Reference

PlantUML supports many diagram types. plantuml.rs is porting them incrementally from the Java original. Below is the current status of each.

## Diagram types

| Diagram type | Status |
|---|---|
| [Sequence Diagram](/language/sequence-diagram/) | ✅ Working |
| [Class Diagram](/language/class-diagram/) | ✅ Working |
| [Activity Diagram](/language/activity-diagram/) | ✅ Working |
| [Use Case Diagram](/language/use-case-diagram/) | ✅ Working |
| [Component Diagram](/language/component-diagram/) | ✅ Working |
| [State Diagram](/language/state-diagram/) | ✅ Working |
| [Object Diagram](/language/object-diagram/) | ✅ Working |
| [Deployment Diagram](/language/deployment-diagram/) | ✅ Working |
| [Timing Diagram](/language/timing-diagram/) | ✅ Working |
| [Mindmap](/language/mindmap/) | ✅ Working |
| [Gantt](/language/gantt/) | ✅ Working |
| [WBS](/language/wbs/) | ✅ Working |
| [JSON](/language/json/) | ✅ Working |
| [YAML](/language/yaml/) | ✅ Working |

## Working now

All 14 diagram types are now implemented and produce valid SVG output. Each diagram type has its own page with syntax examples. The implementations use simplified layout algorithms rather than the full Java Smetana/Graphviz engines — pixel-perfect parity is deferred. 458 tests pass across the workspace.
