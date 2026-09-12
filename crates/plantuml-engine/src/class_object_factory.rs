//! Class and Object diagram factories.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/classdiagram/ClassDiagram.java`
//! - `net/sourceforge/plantuml/classdiagram/ClassDiagramFactory.java`
//! - `net/sourceforge/plantuml/objectdiagram/ObjectDiagramFactory.java`
//!
//! Class and Object diagrams use `@startuml` (not `@startclass`), so they
//! share the UML diagram type set with Sequence, State, etc. The factory
//! checks the source content for class-related keywords to decide if it
//! can handle the source.

use plantuml_core::{DiagramType, PSystemError};
use plantuml_cuca::CucaDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;
/// Factory for Class diagrams.
///
/// Ported from: `ClassDiagramFactory.java`.
pub struct ClassDiagramFactory;

/// Factory for Object diagrams.
///
/// Ported from: `ObjectDiagramFactory.java`.
pub struct ObjectDiagramFactory;

/// Keywords that indicate a class diagram source.
const CLASS_KEYWORDS: &[&str] = &[
    "class ",
    "interface ",
    "abstract ",
    "abstractclass ",
    "enum ",
    "annotation ",
];

/// Keywords that indicate an object diagram source.
const OBJECT_KEYWORDS: &[&str] = &["object "];

impl PSystemFactory for ClassDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Class
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn plantuml_core::diagram::Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        // Check if source contains class-related keywords.
        let has_class_content = lines
            .iter()
            .any(|line| CLASS_KEYWORDS.iter().any(|kw| line.to_lowercase().contains(kw)));

        if !has_class_content {
            return Err(PSystemError::syntax(
                "No class declarations found",
                DiagramType::Class,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = CucaDiagram::from_lines(&line_refs, DiagramType::Class);
        Ok(Box::new(diagram))
    }
}

impl PSystemFactory for ObjectDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Object
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn plantuml_core::diagram::Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        let has_object_content = lines
            .iter()
            .any(|line| OBJECT_KEYWORDS.iter().any(|kw| line.to_lowercase().contains(kw)));

        if !has_object_content {
            return Err(PSystemError::syntax(
                "No object declarations found",
                DiagramType::Object,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = CucaDiagram::from_lines(&line_refs, DiagramType::Object);
        Ok(Box::new(diagram))
    }
}
