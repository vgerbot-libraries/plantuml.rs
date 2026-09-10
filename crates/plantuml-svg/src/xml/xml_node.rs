//! XmlNode — XML element node with attributes and children.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlNode.java`

use crate::xml::xml_leaf::XmlLeaf;
use crate::xml::xml_writer::XmlWriter;
use indexmap::IndexMap;

#[derive(Clone)]
pub enum XmlContent {
    Node(XmlNode),
    Leaf(XmlLeaf),
}

/// An XML element node with a tag name, attributes, and children.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlNode.java`
#[derive(Clone)]
pub struct XmlNode {
    tag_name: String,
    attributes: IndexMap<String, String>,
    children: Vec<XmlContent>,
}

impl XmlNode {
    /// Creates a new `XmlNode` with the given tag name.
    #[must_use]
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: IndexMap::new(),
            children: Vec::new(),
        }
    }

    /// Returns the tag name.
    #[must_use]
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Sets an attribute. If the attribute already exists, it is updated.
    ///
    /// Ported from: `XmlNode.setAttribute(String, String)`.
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name.into(), value.into());
    }

    /// Returns the value of an attribute, if it exists.
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }

    /// Appends a child node.
    ///
    /// Ported from: `XmlNode.appendChild(IElement)`.
    pub fn append_child(&mut self, node: XmlNode) {
        self.children.push(XmlContent::Node(node));
    }

    /// Sets the text content (replaces all children with a single text leaf).
    ///
    /// Ported from: `XmlNode.setTextContent(String)`.
    pub fn set_text_content(&mut self, text: impl Into<String>) {
        self.children.clear();
        self.children.push(XmlContent::Leaf(XmlLeaf::text(text)));
    }

    /// Appends text content.
    ///
    /// Ported from: `XmlNode.appendText(String)`.
    pub fn append_text(&mut self, text: impl Into<String>) {
        self.children
            .push(XmlContent::Leaf(XmlLeaf::text(text)));
    }

    /// Appends a comment.
    ///
    /// Ported from: `XmlNode.appendComment(String)`.
    pub fn append_comment(&mut self, text: impl Into<String>) {
        self.children
            .push(XmlContent::Leaf(XmlLeaf::comment(text)));
    }

    /// Appends a processing instruction.
    ///
    /// Ported from: `XmlNode.appendProcessingInstruction(String, String)`.
    pub fn append_processing_instruction(
        &mut self,
        target: impl Into<String>,
        data: impl Into<String>,
    ) {
        self.children
            .push(XmlContent::Leaf(XmlLeaf::processing_instruction(target, data)));
    }

    /// Appends CDATA content.
    ///
    /// Ported from: `XmlNode.appendCData(String)`.
    pub fn append_cdata(&mut self, text: impl Into<String>) {
        self.children
            .push(XmlContent::Leaf(XmlLeaf::cdata(text)));
    }

    /// Appends raw content (not escaped).
    ///
    /// Ported from: `XmlNode.appendRaw(String)`.
    pub fn append_raw(&mut self, text: impl Into<String>) {
        self.children
            .push(XmlContent::Leaf(XmlLeaf::raw(text)));
    }

    /// Returns the first child element, if any.
    ///
    /// Ported from: `XmlNode.getFirstChild()`.
    #[must_use]
    pub fn first_child(&self) -> Option<&XmlNode> {
        self.children.iter().find_map(|c| match c {
            XmlContent::Node(n) => Some(n),
            XmlContent::Leaf(_) => None,
        })
    }

    /// Returns the first child element, mutably.
    #[must_use]
    pub fn first_child_mut(&mut self) -> Option<&mut XmlNode> {
        self.children.iter_mut().find_map(|c| match c {
            XmlContent::Node(n) => Some(n),
            XmlContent::Leaf(_) => None,
        })
    }

    /// Returns an iterator over child elements.
    pub fn children(&self) -> impl Iterator<Item = &XmlNode> {
        self.children.iter().filter_map(|c| match c {
            XmlContent::Node(n) => Some(n),
            XmlContent::Leaf(_) => None,
        })
    }

    /// Returns a mutable iterator over child elements.
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut XmlNode> {
        self.children.iter_mut().filter_map(|c| match c {
            XmlContent::Node(n) => Some(n),
            XmlContent::Leaf(_) => None,
        })
    }

    /// Clears all children.
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Returns the attributes map.
    #[must_use]
    pub fn attributes(&self) -> &IndexMap<String, String> {
        &self.attributes
    }

    /// Writes this node and its children to the given XmlWriter.
    ///
    /// Ported from: `XmlNode.writeTo(XmlWriter)`.
    pub fn write_to(&self, writer: &mut XmlWriter) {
        writer.start_element(&self.tag_name);
        for (name, value) in &self.attributes {
            writer.attribute(name, value);
        }
        for child in &self.children {
            match child {
                XmlContent::Node(n) => n.write_to(writer),
                XmlContent::Leaf(l) => l.write_to(writer),
            }
        }
        writer.end_element();
    }
}

impl std::fmt::Debug for XmlNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XmlNode")
            .field("tag_name", &self.tag_name)
            .field("attributes", &self.attributes)
            .field("children_count", &self.children.len())
            .finish()
    }
}
