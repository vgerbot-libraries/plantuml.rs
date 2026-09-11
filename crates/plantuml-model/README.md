# plantuml-model

PlantUML entity/relationship model.

Ported from: `net/sourceforge/plantuml/abel/` and `net/sourceforge/plantuml/cucadiagram/` packages.

## Overview

Defines the core entity model used by class diagrams, object diagrams, and other entity-based diagram types. Entities represent classes, interfaces, objects, packages, notes, etc. Links connect entities with arrows, labels, and cardinality.

## Modules

| Module | Description |
|--------|-------------|
| `entity` | `Entity` — the core entity struct (class, interface, package, note, etc.) |
| `entity_factory` | `EntityFactory` — creates and manages entities |
| `link` | `Link` — relationship between entities (association, inheritance, etc.) |
| `leaf_type` | `LeafType` — classification of leaf entities (class, interface, enum, etc.) |
| `group_type` | `GroupType` — grouping types (package, module, folder, etc.) |
| `bodier` | `Bodier` — manages the body content of an entity (fields, methods) |

## Key Exports

- `Entity` — diagram entity with type, display, body, and links
- `Link` — relationship between two entities
- `LeafType` — entity type classification
- `EntityFactory` — entity creation and lookup
- `Bodier` — entity body content manager
- `GroupType` — grouping classification

## License

MIT License (per workspace `LICENSE` file).
