---
title: Use Case Diagram
description: Use case diagram syntax and examples in plantuml.rs
---

# Use Case Diagram

Use case diagrams describe the interactions between actors and the system — use cases, actors, and their relationships.

## Actors and use cases

Declare actors with `actor` and use cases with `usecase`. Link them with arrows.

```plantuml
@startuml
actor User
usecase (Login)
usecase (Logout)
User --> (Login)
User --> (Logout)
@enduml
```

## Use case relationships

Use cases can relate to each other with `-->`, `..>`, and other arrow types.

```plantuml
@startuml
usecase (Shopping)
usecase (Checkout)
usecase (Payment)
(Shopping) --> (Checkout)
(Checkout) --> (Payment)
@enduml
```

## Multiple actors

```plantuml
@startuml
actor Customer
actor Admin
usecase (Manage Orders)
usecase (View Reports)
Customer --> (Manage Orders)
Admin --> (View Reports)
@enduml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
