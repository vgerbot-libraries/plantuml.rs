//! XML DOM for SVG output — custom dependency-free implementation.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/` package.
//! XmlDocument, XmlNode, XmlLeaf, XmlWriter.

mod xml_document;
mod xml_leaf;
mod xml_node;
mod xml_writer;

pub use xml_document::XmlDocument;
pub use xml_leaf::XmlLeaf;
pub use xml_node::XmlNode;
pub use xml_writer::XmlWriter;
