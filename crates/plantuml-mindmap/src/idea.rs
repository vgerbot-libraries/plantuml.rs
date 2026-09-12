//! `Idea` — mindmap tree node.
//!
//! Ported from: `net/sourceforge/plantuml/mindmap/Idea.java` (179 lines).
//!
//! Each idea has a label, level (depth in tree), shape, and children.
//! The tree is built by parsing org-mode `*` prefix or `+`/`-` prefix syntax.

/// Shape of a mindmap node.
///
/// Ported from: `net/sourceforge/plantuml/mindmap/IdeaShape.java`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IdeaShape {
    /// Rounded rectangle box (default).
    #[default]
    Box,
    /// No box — just text (triggered by `_` suffix).
    None,
    /// Empty placeholder (used in WBS for label-less nodes).
    Pseudo,
}

impl IdeaShape {
    /// Returns `None` if the descriptor is `_`, else `Box`.
    ///
    /// Ported from: `IdeaShape.fromDesc(String)`.
    #[must_use]
    pub fn from_desc(desc: Option<&str>) -> Self {
        match desc {
            Some("_") => Self::None,
            _ => Self::Box,
        }
    }
}

/// Direction for mindmap branching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MindMapDirection {
    /// Right side (default).
    #[default]
    Right,
    /// Left side.
    Left,
}

/// A mindmap tree node.
///
/// Ported from: `net/sourceforge/plantuml/mindmap/Idea.java`.
#[derive(Debug, Clone)]
pub struct Idea {
    /// Label text.
    pub label: String,
    /// Depth level (0 = root).
    pub level: usize,
    /// Node shape.
    pub shape: IdeaShape,
    /// Branch direction (for `+`/`-` syntax).
    pub direction: MindMapDirection,
    /// Child ideas.
    pub children: Vec<Self>,
    /// Optional stereotype (`<<stereotype>>`).
    pub stereotype: Option<String>,
    /// Optional background color (`#hex`).
    pub back_color: Option<String>,
}

impl Idea {
    /// Creates a new root idea.
    #[must_use]
    pub const fn new_root(label: String) -> Self {
        Self {
            label,
            level: 0,
            shape: IdeaShape::Box,
            direction: MindMapDirection::Right,
            children: Vec::new(),
            stereotype: None,
            back_color: None,
        }
    }

    /// Creates a child idea with the given level.
    #[must_use]
    pub const fn new_child(label: String, level: usize, shape: IdeaShape, direction: MindMapDirection) -> Self {
        Self {
            label,
            level,
            shape,
            direction,
            children: Vec::new(),
            stereotype: None,
            back_color: None,
        }
    }

    /// Returns `true` if this is a leaf node (no children).
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Returns `true` if this is the root (level 0).
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.level == 0
    }
}

/// Parses org-mode `*` prefix lines into an `Idea` tree.
///
/// Syntax: `* root`, `** child`, `*** grandchild`
/// Optional: `_` suffix for boxless, `[#color]` for background color.
///
/// Returns `None` if no root is found.
#[must_use]
pub fn parse_mindmap_orgmode(lines: &[&str]) -> Option<Idea> {
    parse_tree(lines, false)
}

/// Parses `+`/`-` prefix lines into an `Idea` tree.
///
/// Syntax: `+ root`, `++ child` (right), `-- child` (left)
///
/// Returns `None` if no root is found.
#[must_use]
pub fn parse_mindmap_plus(lines: &[&str]) -> Option<Idea> {
    parse_tree(lines, true)
}

/// Shared tree parsing logic for both org-mode and plus/minus syntax.
///
/// Uses a stack of `Vec<Idea>` where `stack[L]` holds the children being
/// collected at level L. When a node at level L appears, all levels > L
/// are popped and their completed subtrees attached to the last node at
/// level L-1.
fn parse_tree(lines: &[&str], plus_syntax: bool) -> Option<Idea> {
    // stack[L] = list of Ideas at level L that are children of the node at level L-1.
    let mut stack: Vec<Vec<Idea>> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('<') || trimmed.starts_with("style") {
            continue;
        }

        // Parse the prefix markers.
        let (level, rest, is_left) = if plus_syntax {
            let end = trimmed
                .chars()
                .take_while(|&c| c == '+' || c == '-')
                .count();
            if end == 0 {
                continue;
            }
            let left = trimmed.starts_with('-');
            (end.saturating_sub(1), &trimmed[end..], left)
        } else {
            let no_space = trimmed.trim_start();
            let start = trimmed.len() - no_space.len();
            let end = no_space
                .chars()
                .take_while(|&c| c == '*' || c == '#')
                .count();
            if end == 0 {
                continue;
            }
            (end.saturating_sub(1), &trimmed[start + end..], false)
        };

        let (shape, back_color, label) = parse_label_suffix(rest.trim());

        let direction = if plus_syntax && is_left {
            MindMapDirection::Left
        } else {
            MindMapDirection::Right
        };

        let mut idea = Idea::new_child(label, level, shape, direction);
        if let Some(bc) = back_color {
            idea.back_color = Some(bc);
        }

        // Ensure stack has enough levels.
        while stack.len() <= level {
            stack.push(Vec::new());
        }

        // Pop deeper levels and attach their children to this node's siblings.
        // Actually, we need to attach completed subtrees to the parent at level-1.
        while stack.len() > level + 1 {
            let children = stack.pop().unwrap();
            if !children.is_empty() {
                // Attach these children to the last idea at the parent level.
                if let Some(parent_children) = stack.last_mut() {
                    if let Some(parent_idea) = parent_children.last_mut() {
                        parent_idea.children = children;
                    }
                }
            }
        }

        // Push this idea at its level.
        stack[level].push(idea);
    }

    // After all lines, collapse the stack from bottom up.
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

    // The root is the first (and only) idea at level 0.
    stack.into_iter().flatten().next()
}

/// Parses optional `[#color]`, `_` shape suffix, and label from the rest of a line.
fn parse_label_suffix(rest: &str) -> (IdeaShape, Option<String>, String) {
    let mut back_color = None;
    let mut rest = rest;

    // Check for `[#color]` prefix.
    if rest.starts_with('[') {
        if let Some(end) = rest.find(']') {
            let color_str = &rest[1..end];
            if color_str.starts_with('#') {
                back_color = Some(color_str.to_string());
            }
            rest = rest[end + 1..].trim();
        }
    }

    // Check for `_` shape prefix.
    let shape = if rest.starts_with('_') {
        rest = rest[1..].trim_start();
        IdeaShape::None
    } else {
        IdeaShape::Box
    };

    // Extract stereotype `<<...>>` from the end.
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
    fn test_parse_orgmode_simple() {
        let lines = vec!["* root", "** a", "*** a1", "*** a2", "** b"];
        let tree = parse_mindmap_orgmode(&lines).unwrap();
        assert_eq!(tree.label, "root");
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].label, "a");
        assert_eq!(tree.children[0].children.len(), 2);
        assert_eq!(tree.children[0].children[0].label, "a1");
        assert_eq!(tree.children[1].label, "b");
    }

    #[test]
    fn test_parse_plus_syntax() {
        let lines = vec!["+ root", "++ right", "-- left"];
        let tree = parse_mindmap_plus(&lines).unwrap();
        assert_eq!(tree.label, "root");
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].direction, MindMapDirection::Right);
        assert_eq!(tree.children[1].direction, MindMapDirection::Left);
    }

    #[test]
    fn test_boxless_suffix() {
        let lines = vec!["* root", "**_ boxless", "** normal"];
        let tree = parse_mindmap_orgmode(&lines).unwrap();
        assert_eq!(tree.children[0].shape, IdeaShape::None);
        assert_eq!(tree.children[1].shape, IdeaShape::Box);
    }

    #[test]
    fn test_color_suffix() {
        let lines = vec!["* root", "** [#FF0000] red child"];
        let tree = parse_mindmap_orgmode(&lines).unwrap();
        assert_eq!(tree.children[0].back_color, Some("#FF0000".to_string()));
        assert_eq!(tree.children[0].label, "red child");
    }
}
