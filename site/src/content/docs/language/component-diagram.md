---
title: Component Diagram
description: Component diagram syntax and examples in plantuml.rs
---

# Component Diagram

Component diagrams describe the components of a system and their dependencies — components, interfaces, ports, and connectors.

## Declaring components

Declare components with the `component` keyword and bracket syntax, or use other element keywords like `database` and `interface`.

```plantuml
@startuml
component [Web Server]
component [App Server]
database DB
[Web Server] --> [App Server]
[App Server] --> DB
@enduml
```

## Interfaces

Define provided and required interfaces using `()` or the `interface` keyword, then connect them with `use` or arrows.

```plantuml
@startuml
interface "User API" as API
[Client] ..> API : uses
[Service] -- API : provides
@enduml
```

## Packages and groupings

Group components inside packages with the `package` keyword.

```plantuml
@startuml
package "Frontend" {
  component [UI]
}
package "Backend" {
  component [API]
  database Store
}
[UI] --> [API]
[API] --> Store
@enduml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
