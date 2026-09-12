---
title: Deployment Diagram
description: Deployment diagram syntax and examples in plantuml.rs
---

# Deployment Diagram

Deployment diagrams describe the physical deployment of artifacts onto nodes — nodes, artifacts, and communication paths.

## Declaring nodes

Declare nodes with the `node` keyword, and use `database`, `artifact`, or other element keywords for additional deployment targets.

```plantuml
@startuml
node Server
node Client
database DB
Server --> DB
Client --> Server
@enduml
```

## Nested nodes

Place elements inside a node using braces to show containment.

```plantuml
@startuml
node "Application Server" {
  component [Web App]
  database "Cache"
}
node "Database Server" {
  database "Primary DB"
}
[Web App] --> "Cache"
[Web App] --> "Primary DB"
@enduml
```

## Labeled links

Add labels to communication paths to describe the protocol or connection type.

```plantuml
@startuml
node Client
node LoadBalancer
node Server1
node Server2
Client --> LoadBalancer : HTTPS
LoadBalancer --> Server1 : HTTP
LoadBalancer --> Server2 : HTTP
@enduml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
