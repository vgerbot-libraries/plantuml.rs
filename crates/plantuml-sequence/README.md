# plantuml-sequence

Sequence diagram parser and model.

Ported from: `net/sourceforge/plantuml/sequencediagram/` package.

## Overview

Defines the sequence diagram data model: participants, messages, events, and life events. The parser reads PlantUML sequence diagram source and builds a `SequenceDiagram` model that rendering backends consume.

## Modules

| Module | Description |
|--------|-------------|
| `sequence_diagram` | `SequenceDiagram` — the top-level model holding participants and events |
| `participant` | `Participant` — actor/object in the sequence |
| `participant_type` | `ParticipantType` — classification (participant, actor, boundary, etc.) |
| `event` | `Event` — base event trait for sequence diagram events |
| `message` | `Message` — arrow message between participants |
| `life_event` | `LifeEvent` — activation/destruction lifeline events |
| `life_event_type` | `LifeEventType` — classification of life events |

## Key Exports

- `SequenceDiagram` — the diagram model containing participants and events
- `Participant` / `ParticipantType` — participants and their types
- `Event` — base event trait
- `Message` — message/arrow between participants
- `LifeEvent` / `LifeEventType` — lifeline activation/destruction

## License

MIT License (per workspace `LICENSE` file).
