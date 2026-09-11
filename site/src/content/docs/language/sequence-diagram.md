---
title: Sequence Diagram
description: Sequence diagram syntax and examples in plantuml.rs
---

# Sequence Diagram

Sequence diagrams are the first — and currently only — fully implemented diagram type in plantuml.rs. This page documents the supported syntax with live examples rendered to SVG at build time.

## Basic messages

Use `->` for a solid arrow and `-->` for a dashed arrow. Participants are inferred from the messages.

```plantuml
@startuml
Alice -> Bob: hello
Bob --> Alice: hi
@enduml
```

## Declaring participants

Declare participants explicitly to control their left-to-right order. Use `participant`, `actor`, or other keywords.

```plantuml
@startuml
participant Alice
participant Bob
Alice -> Bob: Request
Bob --> Alice: Response
@enduml
```

```plantuml
@startuml
actor User
User -> System: login
System --> User: welcome
@enduml
```

## Self messages

A participant can send a message to itself.

```plantuml
@startuml
Alice -> Alice: self message
@enduml
```

## Notes

Add notes with the `note` keyword. Place them `left of`, `right of`, or `over` a participant.

```plantuml
@startuml
Alice -> Bob: hello
note right of Alice: says hello
Bob --> Alice: hi
note right of Bob: replies
@enduml
```

Notes can span multiple participants with `note over`:

```plantuml
@startuml
Alice -> Bob: hello
note over Alice, Bob: both
@enduml
```

## Activation and deactivation

Use `activate` and `deactivate` to show lifeline activation.

```plantuml
@startuml
Alice -> Bob: request
activate Bob
Bob --> Alice: response
deactivate Bob
@enduml
```

## Grouping

Group messages with `group`/`end`, or use the `alt`/`else`/`end` and `loop`/`end` constructs.

```plantuml
@startuml
group Authentication
Alice -> Bob: credentials
Bob --> Alice: token
end
@enduml
```

### alt / else

```plantuml
@startuml
alt success
Alice -> Bob: ok
else failure
Alice -> Bob: fail
end
@enduml
```

### loop

```plantuml
@startuml
loop until done
Alice -> Bob: check
Bob --> Alice: not yet
end
@enduml
```

## Dividers

Use `==` to insert a divider between groups of messages.

```plantuml
@startuml
Alice -> Bob: step 1
== Phase 2 ==
Bob -> Alice: step 2
@enduml
```

## Automatic numbering

Use `autonumber` to automatically number messages.

```plantuml
@startuml
autonumber
Alice -> Bob: first
Bob --> Alice: second
@enduml
```

## Title

Set a title with the `title` keyword.

```plantuml
@startuml
title My Sequence
Alice -> Bob: hello
@enduml
```

## Styling with skinparam

Customize appearance with `skinparam`. For example, set the background color.

```plantuml
@startuml
skinparam backgroundColor #EEF
Alice -> Bob: styled message
@enduml
```

## Colored messages

Specify arrow color inline with `-[#color]`.

```plantuml
@startuml
Alice -[#red]-> Bob: red message
@enduml
```
