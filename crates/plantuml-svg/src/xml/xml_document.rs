//! XmlDocument — XML document root.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlDocument.java`

use crate::xml::xml_node::XmlNode;
use crate::xml::xml_writer::XmlWriter;

/// An XML document holding a root element.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlDocument.java`
pub struct XmlDocument {
    root: Option<XmlNode>,
}

impl XmlDocument {
    /// Creates a new empty `XmlDocument`.
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Creates a new element with the given tag name.
    ///
    /// Ported from: `XmlDocument.createElement(String)`.
    #[must_use]
    pub fn create_element(&self, tag_name: impl Into<String>) -> XmlNode {
        XmlNode::new(tag_name)
    }

    /// Sets the root element.
    pub fn set_root(&mut self, root: XmlNode) {
        self.root = Some(root);
    }

    /// Returns the root element, if any.
    #[must_use]
    pub fn root(&self) -> Option<&XmlNode> {
        self.root.as_ref()
    }

    /// Returns the root element mutably, if any.
    #[must_use]
    pub fn root_mut(&mut self) -> Option<&mut XmlNode> {
        self.root.as_mut()
    }

    /// Serializes the document to an XML string.
    ///
    /// Ported from: `XmlDocument.toXml(int)` where the argument is indent spaces.
    #[must_use]
    pub fn to_xml(&self, indent_spaces: usize) -> String {
        let mut writer = XmlWriter::new(indent_spaces);
        if let Some(root) = &self.root {
            root.write_to(&mut writer);
        }
        writer.into_string()
    }
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}
