---
title: Class Diagram
description: Class diagram syntax and examples in plantuml.rs
---

# Class Diagram

Class diagrams describe the static structure of a system — classes, interfaces, attributes, methods, and relationships (inheritance, association, composition, etc.).

## Declaring classes

Declare classes, interfaces, and abstract classes with the corresponding keyword.

```plantuml
@startuml
class Alice
interface Bob
abstract Foo
@enduml
```

## Relationships

Draw relationships between classes with arrow operators: `-->` for association, `--|>` for inheritance, `*--` for composition, `o--` for aggregation.

```plantuml
@startuml
class Animal
class Dog
class Cat
Animal <|-- Dog
Animal <|-- Cat
@enduml
```

```plantuml
@startuml
interface Shape
class Circle
class Square
Shape <|.. Circle
Shape <|.. Square
@enduml
```

## Class body

Define fields and methods inside a class body. Use `+` for public, `-` for private, `#` for protected visibility.

```plantuml
@startuml
class Foo {
  +field: int
  -privateField: String
  #protectedField: bool
  +method(): void
  -helper(): int
}
@enduml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.
