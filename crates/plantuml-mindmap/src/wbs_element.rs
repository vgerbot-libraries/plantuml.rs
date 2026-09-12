//! `WElement` — WBS tree node.
//!
//! Ported from: `net/sourceforge/plantuml/wbs/WElement.java`,
//! `net/sourceforge/plantuml/wbs/Fork.java`, `net/sourceforge/plantuml/wbs/ITF.java`.

use crate::idea::IdeaShape;

/// A WBS tree element (node).
///
/// Ported from: `net/sourceforge/plantuml/wbs/WElement.java`.
#[derive(Debug, Clone)]
pub struct WElement {
    /// Label text.
    pub label: String,
    /// Depth level (0 = root).
    pub level: usize,
    /// Node shape.
    pub shape: IdeaShape,
    /// Child elements.
    pub children: Vec<Self>,
    /// Optional stereotype.
    pub stereotype: Option<String>,
    /// Optional background color.
    pub back_color: Option<String>,
}

impl WElement {
    /// Creates a new root element.
    #[must_use]
    pub const fn new_root(label: String) -> Self {
        Self {
            label,
            level: 0,
            shape: IdeaShape::Box,
            children: Vec::new(),
            stereotype: None,
            back_color: None,
        }
    }

    /// Creates a child element.
    #[must_use]
    pub const fn new_child(label: String, level: usize) -> Self {
        Self {
            label,
            level,
            shape: IdeaShape::Box,
            children: Vec::new(),
            stereotype: None,
            back_color: None,
        }
    }

    /// Returns `true` if this is a leaf (no children).
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// Parses `*` prefix lines into a `WElement` tree.
///
/// Syntax: `* root`, `** child`, `*** grandchild`
/// Optional: `_` suffix for boxless, `[#color]` for background.
///
/// Ported from: `WBSDiagram.addIdea()` + `WBSDiagram.getSmartLevel()`.
#[must_use]
pub fn parse_wbs_tree(lines: &[&str]) -> Option<WElement> {
    let mut stack: Vec<Vec<WElement>> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('<') || trimmed.starts_with("style") {
            continue;
        }

        // Parse `*` prefix markers.
        let marker_end = trimmed
            .chars()
            .take_while(|&c| c == '*')
            .count();
        if marker_end == 0 {
            continue;
        }

        let level = marker_end.saturating_sub(1);
        let rest = trimmed[marker_end..].trim();

        let (shape, back_color, label) = parse_wbs_label(rest);

        let mut element = WElement::new_child(label, level);
        element.shape = shape;
        element.back_color = back_color;

        // Ensure stack has enough levels.
        while stack.len() <= level {
            stack.push(Vec::new());
        }

        // Pop deeper levels and attach children to parent.
        while stack.len() > level + 1 {
            let children = stack.pop().unwrap();
            if !children.is_empty() {
                if let Some(parent_children) = stack.last_mut() {
                    if let Some(parent_idea) = parent_children.last_mut() {
                        parent_idea.children = children;
                    }
                }
            }
        }

        stack[level].push(element);
    }

    // Collapse stack from bottom up.
    while stack.len() > 1 {
        let children = stack.pop().unwrap();
        if !children.is_empty() {
            if let Some(parent_children) = stack.last_mut() {
                if let Some(parent_idea) = parent_children.last_mut() {
                    parent_idea.children = children;
                }
            }
        }
    }

    stack.into_iter().flatten().next()
}

/// Parses optional `[#color]`, `_` shape, and label from WBS line rest.
fn parse_wbs_label(rest: &str) -> (IdeaShape, Option<String>, String) {
    let mut back_color = None;
    let mut rest = rest;

    if rest.starts_with('[') {
        if let Some(end) = rest.find(']') {
            let color_str = &rest[1..end];
            if color_str.starts_with('#') {
                back_color = Some(color_str.to_string());
            }
            rest = rest[end + 1..].trim();
        }
    }

    let shape = if rest.starts_with('_') {
        rest = rest[1..].trim_start();
        IdeaShape::None
    } else {
        IdeaShape::Box
    };

    // Extract stereotype.
    let label = if let Some(start) = rest.rfind("<<") {
        if let Some(end) = rest.rfind(">>") {
            if end > start {
                rest[..start].trim().to_string()
            } else {
                rest.to_string()
            }
        } else {
            rest.to_string()
        }
    } else {
        rest.to_string()
    };

    (shape, back_color, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wbs_simple() {
        let lines = vec!["* Project", "** Planning", "*** Define scope", "** Implementation"];
        let tree = parse_wbs_tree(&lines).unwrap();
        assert_eq!(tree.label, "Project");
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].label, "Planning");
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(tree.children[1].label, "Implementation");
    }

    #[test]
    fn test_parse_wbs_boxless() {
        let lines = vec!["* root", "**_ boxless"];
        let tree = parse_wbs_tree(&lines).unwrap();
        assert_eq!(tree.children[0].shape, IdeaShape::None);
    }
}
