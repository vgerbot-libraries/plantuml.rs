//! XmlLeaf — XML leaf content (text, comment, PI, CDATA, raw).
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlLeaf.java`

use crate::xml::xml_writer::XmlWriter;

/// The kind of leaf content.
///
/// Ported from: `XmlLeaf.Kind` enum.
#[derive(Debug, Clone)]
pub enum LeafKind {
    Text,
    Comment,
    ProcessingInstruction,
    CData,
    Raw,
}

/// A leaf node in the XML tree — text, comment, processing instruction, CDATA, or raw.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlLeaf.java`
#[derive(Debug, Clone)]
pub struct XmlLeaf {
    kind: LeafKind,
    text: String,
    data: Option<String>,
}

impl XmlLeaf {
    /// Creates a text leaf.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: LeafKind::Text,
            text: text.into(),
            data: None,
        }
    }

    /// Creates a comment leaf.
    #[must_use]
    pub fn comment(text: impl Into<String>) -> Self {
        Self {
            kind: LeafKind::Comment,
            text: text.into(),
            data: None,
        }
    }

    /// Creates a processing instruction leaf.
    #[must_use]
    pub fn processing_instruction(target: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            kind: LeafKind::ProcessingInstruction,
            text: target.into(),
            data: Some(data.into()),
        }
    }

    /// Creates a CDATA leaf.
    #[must_use]
    pub fn cdata(text: impl Into<String>) -> Self {
        Self {
            kind: LeafKind::CData,
            text: text.into(),
            data: None,
        }
    }

    /// Creates a raw leaf (not escaped).
    #[must_use]
    pub fn raw(text: impl Into<String>) -> Self {
        Self {
            kind: LeafKind::Raw,
            text: text.into(),
            data: None,
        }
    }

    /// Writes this leaf to the given XmlWriter.
    ///
    /// Ported from: `XmlLeaf.writeTo(XmlWriter)`.
    pub fn write_to(&self, writer: &mut XmlWriter) {
        match self.kind {
            LeafKind::Text => writer.text(&self.text),
            LeafKind::Comment => writer.comment(&self.text),
            LeafKind::ProcessingInstruction => {
                if let Some(data) = &self.data {
                    writer.processing_instruction(&self.text, data);
                }
            }
            LeafKind::CData => writer.cdata(&self.text),
            LeafKind::Raw => writer.raw(&self.text),
        }
    }
}
