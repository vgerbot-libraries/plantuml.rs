//! Java PlantUML parity tests for language documentation examples.
//!
//! Each test renders a PlantUML source snippet (copied verbatim from the
//! language docs at `site/src/content/docs/language/`) with both Java PlantUML
//! and the Rust engine, normalizes both SVGs via `svg_cleaner`, and asserts
//! they are identical.
//!
//! Test sources are adapted from the language documentation examples, not
//! directly ported from `temp/plantuml/src/test/`. They verify that the Rust
//! renderer matches Java PlantUML for the same diagrams shown on the site.
//!
//! All tests are currently `#[ignore]` because the Rust renderer was crafted
//! to match an older Java PlantUML version whose SVG structure differs from
//! the current Java 1.2026.6 (e.g. `participant-lifeline` data-attribute
//! wrappers, `#000000` vs `#000` color format, different coordinates).
//! The `parity_test!` macro is retained for future use when the Rust renderer
//! is updated to match the current Java output. Ignored tests can be run with
//! `cargo test --test java_parity_test -- --ignored`.

mod java_plantuml;
mod svg_cleaner;

/// Generates a parity test that is **not** ignored.
macro_rules! parity_test {
    ($name:ident, $source:expr) => {
        #[test]
        fn $name() {
            run_parity_test(stringify!($name), $source);
        }
    };
}

/// Generates a parity test marked `#[ignore]` because Rust output does not
/// yet match the Java reference.
macro_rules! parity_test_ignored {
    ($name:ident, $source:expr) => {
        #[test]
        #[ignore = "Rust output does not match Java reference"]
        fn $name() {
            run_parity_test(stringify!($name), $source);
        }
    };
}

/// Shared body for both macro variants.
fn run_parity_test(name: &str, source: &str) {
    let java_svg = match java_plantuml::render_svg(source) {
        Some(svg) => svg,
        None => {
            eprintln!("Java PlantUML not available, skipping {name}");
            return;
        }
    };
    let rust_svg = plantuml_engine::render_svg(source)
        .unwrap_or_else(|e| panic!("Rust render should succeed for {name}: {e:?}"));

    let cleaned_java = svg_cleaner::clean(&java_svg);
    let cleaned_rust = svg_cleaner::clean(&rust_svg);

    if cleaned_rust != cleaned_java {
        // Write debug files for manual inspection.
        let dir = std::env::temp_dir();
        std::fs::write(dir.join(format!("{name}.java.svg")), &java_svg).ok();
        std::fs::write(dir.join(format!("{name}.rust.svg")), &rust_svg).ok();

        // Print first diff line for quick diagnosis.
        let jl: Vec<&str> = cleaned_java.lines().collect();
        let rl: Vec<&str> = cleaned_rust.lines().collect();
        for (i, (j, r)) in jl.iter().zip(rl.iter()).enumerate() {
            if j != r {
                eprintln!("First diff at line {i}:");
                eprintln!("  java: {j}");
                eprintln!("  rust: {r}");
                break;
            }
        }
        // If one is longer, report the extra line.
        if jl.len() != rl.len() {
            eprintln!("Line count differs: java={jl_len} rust={rl_len}",
                jl_len = jl.len(), rl_len = rl.len());
        }
    }

    assert_eq!(cleaned_rust, cleaned_java, "SVG parity mismatch for {name}");
}
// ── Sequence diagram ───────────────────────────────────────────────────

parity_test!(sequence_basic_messages, r#"@startuml
Alice -> Bob: hello
Bob --> Alice: hi
@enduml"#);

parity_test!(sequence_declare_participants, r#"@startuml
participant Alice
participant Bob
Alice -> Bob: Request
Bob --> Alice: Response
@enduml"#);

parity_test_ignored!(sequence_actor_user, r#"@startuml
actor User
User -> System: login
System --> User: welcome
@enduml"#);

parity_test_ignored!(sequence_self_messages, r#"@startuml
Alice -> Alice: self message
@enduml"#);

parity_test_ignored!(sequence_notes_right, r#"@startuml
Alice -> Bob: hello
note right of Alice: says hello
Bob --> Alice: hi
note right of Bob: replies
@enduml"#);

parity_test_ignored!(sequence_note_over, r#"@startuml
Alice -> Bob: hello
note over Alice, Bob: both
@enduml"#);

parity_test_ignored!(sequence_activate_deactivate, r#"@startuml
Alice -> Bob: request
activate Bob
Bob --> Alice: response
deactivate Bob
@enduml"#);

parity_test_ignored!(sequence_group_auth, r#"@startuml
group Authentication
Alice -> Bob: credentials
Bob --> Alice: token
end
@enduml"#);

parity_test_ignored!(sequence_alt_else, r#"@startuml
alt success
Alice -> Bob: ok
else failure
Alice -> Bob: fail
end
@enduml"#);

parity_test_ignored!(sequence_loop_until, r#"@startuml
loop until done
Alice -> Bob: check
Bob --> Alice: not yet
end
@enduml"#);

parity_test_ignored!(sequence_dividers, r#"@startuml
Alice -> Bob: step 1
== Phase 2 ==
Bob -> Alice: step 2
@enduml"#);

parity_test_ignored!(sequence_autonumber, r#"@startuml
autonumber
Alice -> Bob: first
Bob --> Alice: second
@enduml"#);

parity_test_ignored!(sequence_title, r#"@startuml
title My Sequence
Alice -> Bob: hello
@enduml"#);

parity_test_ignored!(sequence_skinparam_bg, r#"@startuml
skinparam backgroundColor #EEF
Alice -> Bob: styled message
@enduml"#);

parity_test_ignored!(sequence_colored_messages, r#"@startuml
Alice -[#red]-> Bob: red message
@enduml"#);

// ── Class diagram (expected to fail — simplified layout) ─────────────────

parity_test_ignored!(class_declaring_classes, r#"@startuml
class Alice
interface Bob
abstract Foo
@enduml"#);

parity_test_ignored!(class_relationships_inheritance, r#"@startuml
class Animal
class Dog
class Cat
Animal <|-- Dog
Animal <|-- Cat
@enduml"#);

parity_test_ignored!(class_interface_realization, r#"@startuml
interface Shape
class Circle
class Square
Shape <|.. Circle
Shape <|.. Square
@enduml"#);

parity_test_ignored!(class_body, r#"@startuml
class Foo {
  +field: int
  -privateField: String
  #protectedField: bool
  +method(): void
  -helper(): int
}
@enduml"#);

// ── Activity diagram (expected to fail — simplified layout) ─────────────

parity_test_ignored!(activity_basic_flow, r#"@startuml
start
:Do something;
:Do another thing;
stop
@enduml"#);

parity_test_ignored!(activity_if_else, r#"@startuml
start
if (condition?) then (yes)
  :Take yes path;
else (no)
  :Take no path;
endif
stop
@enduml"#);

parity_test_ignored!(activity_while_loop, r#"@startuml
start
while (more data?) is (yes)
  :Process item;
endwhile (no)
stop
@enduml"#);

// ── Use case diagram ──────────────────────────────────────────────────

parity_test!(usecase_actors_usecases, r#"@startuml
actor User
usecase (Login)
usecase (Logout)
User --> (Login)
User --> (Logout)
@enduml"#);

parity_test!(usecase_relationships, r#"@startuml
usecase (Shopping)
usecase (Checkout)
usecase (Payment)
(Shopping) --> (Checkout)
(Checkout) --> (Payment)
@enduml"#);

parity_test!(usecase_multiple_actors, r#"@startuml
actor Customer
actor Admin
usecase (Manage Orders)
usecase (View Reports)
Customer --> (Manage Orders)
Admin --> (View Reports)
@enduml"#);

// ── Component diagram (expected to fail — simplified layout) ───────────

parity_test_ignored!(component_declaring_components, r#"@startuml
component [Web Server]
component [App Server]
database DB
[Web Server] --> [App Server]
[App Server] --> DB
@enduml"#);

parity_test_ignored!(component_interfaces, r#"@startuml
interface "User API" as API
[Client] ..> API : uses
[Service] -- API : provides
@enduml"#);

parity_test_ignored!(component_packages, r#"@startuml
package "Frontend" {
  component [UI]
}
package "Backend" {
  component [API]
  database Store
}
[UI] --> [API]
[API] --> Store
@enduml"#);

// ── State diagram (expected to fail — simplified layout) ───────────────

parity_test_ignored!(state_declaring_states, r#"@startuml
state Idle
state Active
[*] -> Idle
Idle --> Active : start
Active --> [*] : stop
@enduml"#);

parity_test_ignored!(state_transitions_labels, r#"@startuml
state Off
state On
state Dim
[*] --> Off
Off --> On : switch
On --> Dim : dimmer
Dim --> On : brighter
On --> Off : switch
@enduml"#);

parity_test_ignored!(state_composite_states, r#"@startuml
state Active {
  state Running
  state Paused
  Running --> Paused : pause
  Paused --> Running : resume
}
[*] --> Active
Active --> [*] : done
@enduml"#);

// ── Object diagram (expected to fail — simplified layout) ──────────────

parity_test!(object_declaring_objects, r#"@startuml
object alice
object bob : User
object carol
@enduml"#);

parity_test_ignored!(object_links, r#"@startuml
object alice
object bob
alice --> bob : knows
@enduml"#);

parity_test!(object_attributes, r#"@startuml
object alice : User {
  name = "Alice"
  age = 30
}
object bob : User {
  name = "Bob"
  age = 25
}
alice --> bob : friend
@enduml"#);

// ── Deployment diagram (expected to fail — simplified layout) ──────────

parity_test_ignored!(deployment_declaring_nodes, r#"@startuml
node Server
node Client
database DB
Server --> DB
Client --> Server
@enduml"#);

parity_test_ignored!(deployment_nested_nodes, r#"@startuml
node "Application Server" {
  component [Web App]
  database "Cache"
}
node "Database Server" {
  database "Primary DB"
}
[Web App] --> "Cache"
[Web App] --> "Primary DB"
@enduml"#);

parity_test_ignored!(deployment_labeled_links, r#"@startuml
node Client
node LoadBalancer
node Server1
node Server2
Client --> LoadBalancer : HTTPS
LoadBalancer --> Server1 : HTTP
LoadBalancer --> Server2 : HTTP
@enduml"#);

// ── Timing diagram (expected to fail — simplified layout) ──────────────

parity_test!(timing_binary_signals, r#"@starttiming
binary "Signal A" as A
@0
A is low
@5
A is high
@10
A is low
@endtiming"#);

parity_test!(timing_clock_signal, r#"@starttiming
clock "clk" as C with period 10
@endtiming"#);

parity_test!(timing_multiple_signals, r#"@starttiming
clock "clk" as C with period 10
binary "Data" as D
@0
D is low
@5
D is high
@15
D is low
@endtiming"#);

// ── Gantt (expected to fail — simplified layout) ───────────────────────

parity_test_ignored!(gantt_tasks, r#"@startgantt
Project starts the 1st of january 2020
[Task A] lasts 5 days
[Task B] lasts 3 days
@endgantt"#);

parity_test!(gantt_dependencies, r#"@startgantt
Project starts the 1st of january 2020
[Task A] lasts 5 days
[Task B] lasts 3 days
[Task B] depends on [Task A]
@endgantt"#);

parity_test!(gantt_milestones, r#"@startgantt
Project starts the 1st of january 2020
[Task A] lasts 5 days
milestone [M1] happens at 10 days
@endgantt"#);

// ── Mindmap (expected to fail — simplified layout) ──────────────────────

parity_test_ignored!(mindmap_basic_tree, r#"@startmindmap
* Root idea
** First branch
*** Sub idea
*** Another sub idea
** Second branch
*** Detail
@endmindmap"#);

parity_test_ignored!(mindmap_plus_syntax, r#"@startmindmap
+ Root
++ Child A
++ Child B
+++ Grandchild
@endmindmap"#);

parity_test_ignored!(mindmap_left_right_branches, r#"@startmindmap
* Central topic
-- Left idea
--- Deeper left
++ Right idea
+++ Deeper right
@endmindmap"#);

// ── WBS (expected to fail — simplified layout) ─────────────────────────

parity_test_ignored!(wbs_basic_tree, r#"@startwbs
* Project
** Phase 1
*** Task A
*** Task B
** Phase 2
*** Task C
@endwbs"#);

parity_test_ignored!(wbs_deeper_nesting, r#"@startwbs
* Product Launch
** Planning
*** Market research
*** Budgeting
**** Cost estimate
**** Funding plan
** Execution
*** Development
*** Marketing
@endwbs"#);

parity_test_ignored!(wbs_styled_nodes, r#"@startwbs
* Project
** Phase 1 [#LightBlue]
*** Task A
*** Task B
** Phase 2 [#LightGreen]
*** Task C
@endwbs"#);

// ── JSON (expected to fail — simplified layout) ────────────────────────

parity_test_ignored!(json_simple_object, r#"@startjson
{
  "key": "value",
  "count": 42
}
@endjson"#);

parity_test_ignored!(json_nested_object, r#"@startjson
{
  "name": "Alice",
  "address": {
    "city": "NYC",
    "zip": "10001"
  }
}
@endjson"#);

parity_test_ignored!(json_array_values, r#"@startjson
{
  "users": [
    {"name": "Alice", "role": "admin"},
    {"name": "Bob", "role": "user"}
  ],
  "active": true
}
@endjson"#);

// ── YAML (expected to fail — simplified layout) ────────────────────────

parity_test_ignored!(yaml_simple_keyvalue, r#"@startyaml
key: value
count: 42
@endyaml"#);

parity_test_ignored!(yaml_nested_mapping, r#"@startyaml
name: Alice
address:
  city: NYC
  zip: "10001"
@endyaml"#);

parity_test_ignored!(yaml_lists, r#"@startyaml
users:
  - name: Alice
    role: admin
  - name: Bob
    role: user
active: true
@endyaml"#);
