//! `PSystemCommandFactory` — generic command-based diagram factory.
//!
//! Ported from: net/sourceforge/plantuml/command/PSystemCommandFactory.java
//!
//! This is the standard factory pattern for most diagram types: a list of
//! `Command<D>` implementations are tried in order against each source line.
//! The first matching command executes against the diagram, accumulating
//! state until all lines are consumed.

use plantuml_command::{BlocLines, Command, CommandControl, ParserPass};
use plantuml_core::{Diagram, DiagramType, PSystemError};

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Callback that creates an empty diagram `D`.
pub type CreateEmptyDiagram<D> = fn() -> D;

/// A factory that creates diagrams by executing commands against source lines.
///
/// Generic over `D: Diagram`. Each diagram type (class, state, component, etc.)
/// creates a `PSystemCommandFactory` with its own command list and empty-diagram
/// constructor.
///
/// Ported from: net/sourceforge/plantuml/command/PSystemCommandFactory.java
pub struct PSystemCommandFactory<D: Diagram + 'static> {
    diagram_type: DiagramType,
    commands: Vec<Box<dyn Command<D>>>,
    create_empty: CreateEmptyDiagram<D>,
}

impl<D: Diagram + 'static> PSystemCommandFactory<D> {
    /// Creates a new `PSystemCommandFactory` with the given diagram type,
    /// empty-diagram constructor, and command list.
    ///
    /// Ported from: `PSystemCommandFactory(DiagramType)` constructor +
    /// `initCommandsList`.
    #[must_use]
    pub fn new(
        diagram_type: DiagramType,
        create_empty: CreateEmptyDiagram<D>,
        commands: Vec<Box<dyn Command<D>>>,
    ) -> Self {
        Self {
            diagram_type,
            commands,
            create_empty,
        }
    }

    /// Returns the diagram type.
    #[must_use]
    pub const fn diagram_type(&self) -> DiagramType {
        self.diagram_type
    }
}

impl<D: Diagram + 'static> PSystemFactory for PSystemCommandFactory<D> {
    fn get_diagram_type(&self) -> DiagramType {
        self.diagram_type
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let mut diagram = (self.create_empty)();

        for pass in [ParserPass::One, ParserPass::Two, ParserPass::Three] {
            for line in source.body_iter() {
                let s = line.get_string().trim();
                if s.starts_with("@end") || s.starts_with("\\end") {
                    break;
                }

                let single = BlocLines::single(line);
                for cmd in &self.commands {
                    if !cmd.is_eligible_for(pass) {
                        continue;
                    }
                    match cmd.is_valid(&single) {
                        CommandControl::Ok => {
                            match cmd.execute(&mut diagram, &single, pass) {
                                Ok(result) => {
                                    if !result.is_ok() {
                                        return Err(PSystemError::execution(
                                            result.get_error().unwrap_or("Execution error"),
                                            self.diagram_type,
                                        ));
                                    }
                                }
                                Err(_) => {
                                    return Err(PSystemError::execution(
                                        "Color error",
                                        self.diagram_type,
                                    ));
                                }
                            }
                            break;
                        }
                        CommandControl::OkPartial | CommandControl::NotOk => {}
                    }
                }
            }
        }

        Ok(Box::new(diagram))
    }
}
