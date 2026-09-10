//! XmlWriter — streaming XML serializer.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlWriter.java`

/// A streaming XML serializer with stack-based element tracking.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/XmlWriter.java`
pub struct XmlWriter {
    output: String,
    open_tags: Vec<String>,
    indent_spaces: usize,
    has_children: Vec<bool>,
    pending_attributes: Vec<(String, String)>,
}

impl XmlWriter {
    /// Creates a new `XmlWriter` with the given indentation.
    ///
    /// `indent_spaces = 0` means no indentation or newlines.
    #[must_use]
    pub fn new(indent_spaces: usize) -> Self {
        Self {
            output: String::new(),
            open_tags: Vec::new(),
            indent_spaces,
            has_children: Vec::new(),
            pending_attributes: Vec::new(),
        }
    }

    /// Starts a new element.
    ///
    /// Ported from: `XmlWriter.startElement(String)`.
    pub fn start_element(&mut self, tag: &str) {
        self.flush_open_tag();
        self.write_indent();
        self.output.push('<');
        self.output.push_str(tag);
        self.open_tags.push(tag.to_string());
        self.has_children.push(false);
    }

    /// Adds an attribute to the currently open element.
    ///
    /// Ported from: `XmlWriter.attribute(String, String)`.
    pub fn attribute(&mut self, name: &str, value: &str) {
        self.pending_attributes
            .push((name.to_string(), value.to_string()));
    }

    /// Adds text content.
    ///
    /// Ported from: `XmlWriter.text(String)`.
    pub fn text(&mut self, text: &str) {
        self.flush_open_tag();
        self.output.push_str(&escape_text(text));
    }

    /// Adds a comment.
    ///
    /// Ported from: `XmlWriter.comment(String)`.
    pub fn comment(&mut self, text: &str) {
        self.flush_open_tag();
        self.write_indent();
        self.output.push_str("<!--");
        self.output.push_str(&defang_comment(text));
        self.output.push_str("-->");
        if self.indent_spaces > 0 {
            self.output.push('\n');
        }
    }

    /// Adds a processing instruction.
    ///
    /// Ported from: `XmlWriter.processingInstruction(String, String)`.
    pub fn processing_instruction(&mut self, target: &str, data: &str) {
        self.flush_open_tag();
        self.output.push_str("<?");
        self.output.push_str(target);
        self.output.push(' ');
        self.output.push_str(data);
        self.output.push_str("?>");
        if self.indent_spaces > 0 {
            self.output.push('\n');
        }
    }

    /// Adds CDATA content.
    ///
    /// Ported from: `XmlWriter.cdata(String)`.
    pub fn cdata(&mut self, text: &str) {
        self.flush_open_tag();
        // Split ]]> across sections
        let parts: Vec<&str> = text.split("]]>").collect();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                self.output.push_str("]]><![CDATA[>");
            }
            self.output.push_str("<![CDATA[");
            self.output.push_str(part);
            self.output.push_str("]]>");
        }
    }

    /// Adds raw content (not escaped).
    ///
    /// Ported from: `XmlWriter.raw(String)`.
    pub fn raw(&mut self, text: &str) {
        self.flush_open_tag();
        self.output.push_str(text);
    }

    /// Ends the current element.
    ///
    /// Ported from: `XmlWriter.endElement()`.
    pub fn end_element(&mut self) {
        if let Some(tag) = self.open_tags.pop() {
            let had_children = self.has_children.pop().unwrap_or(false);

            if !had_children {
                // Self-closing tag
                self.flush_attributes();
                self.output.push_str("/>");
                if self.indent_spaces > 0 {
                    self.output.push('\n');
                }
            } else {
                // Close tag
                self.write_indent();
                self.output.push_str("</");
                self.output.push_str(&tag);
                self.output.push('>');
                if self.indent_spaces > 0 {
                    self.output.push('\n');
                }
            }
        }
    }

    /// Returns the serialized XML string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.output
    }

    /// Returns a reference to the serialized XML string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.output
    }

    // -- Internal helpers --

    fn flush_open_tag(&mut self) {
        if let Some(last) = self.has_children.last_mut() {
            if !*last {
                *last = true;
                self.flush_attributes();
                self.output.push('>');
                if self.indent_spaces > 0 {
                    self.output.push('\n');
                }
            }
        }
    }

    fn flush_attributes(&mut self) {
        for (name, value) in self.pending_attributes.drain(..) {
            self.output.push(' ');
            self.output.push_str(&name);
            self.output.push_str("=\"");
            self.output.push_str(&escape_attr(&value));
            self.output.push('"');
        }
    }

    fn write_indent(&mut self) {
        if self.indent_spaces > 0 {
            let depth = self.open_tags.len();
            for _ in 0..depth * self.indent_spaces {
                self.output.push(' ');
            }
        }
    }
}

/// Escapes text content: `&` and `<` only.
fn escape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            _ => result.push(c),
        }
    }
    result
}

/// Escapes attribute values: `&`, `<`, and `"`.
fn escape_attr(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

/// Defangs `--` in comments to `- -`.
fn defang_comment(s: &str) -> String {
    s.replace("--", "- -")
}
