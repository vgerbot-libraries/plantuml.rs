//! Diagram type enumeration and `@start` keyword dispatch.
//!
//! Ported from: net/sourceforge/plantuml/core/DiagramType.java

use std::collections::HashSet;

/// The diagram type, determined from the `@start<keyword>` directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramType {
    Sequence,
    State,
    Class,
    Object,
    Activity,
    Description,
    Composite,
    Timing,
    Help,
    Bpm,
    Ditaa,
    Dot,
    Jcckit,
    Salt,
    Flow,
    Creole,
    Math,
    Latex,
    Definition,
    Gantt,
    Chronology,
    Nwdiag,
    Mindmap,
    Wbs,
    Wire,
    Json,
    Git,
    Board,
    Yaml,
    Hcl,
    Ebnf,
    Regex,
    Files,
    ChenEer,
    Chart,
    Packet,
    Sprites,
    Crash,
    Unknown,
}

impl DiagramType {
    /// `true` for the legacy UML diagram types.
    #[must_use]
    pub fn is_legacy_uml(&self) -> bool {
        matches!(
            self,
            Self::Sequence
                | Self::State
                | Self::Class
                | Self::Object
                | Self::Activity
                | Self::Description
                | Self::Composite
                | Self::Timing
                | Self::Help
                | Self::Sprites
        )
    }

    /// Human-readable name (e.g. `"sequence"`, `"component"`).
    #[must_use]
    pub fn human_readable_name(&self) -> &'static str {
        if *self == Self::Description {
            return "component";
        }
        self.name_lower()
    }

    const fn name_lower(&self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::State => "state",
            Self::Class => "class",
            Self::Object => "object",
            Self::Activity => "activity",
            Self::Description => "description",
            Self::Composite => "composite",
            Self::Timing => "timing",
            Self::Help => "help",
            Self::Bpm => "bpm",
            Self::Ditaa => "ditaa",
            Self::Dot => "dot",
            Self::Jcckit => "jcckit",
            Self::Salt => "salt",
            Self::Flow => "flow",
            Self::Creole => "creole",
            Self::Math => "math",
            Self::Latex => "latex",
            Self::Definition => "definition",
            Self::Gantt => "gantt",
            Self::Chronology => "chronology",
            Self::Nwdiag => "nwdiag",
            Self::Mindmap => "mindmap",
            Self::Wbs => "wbs",
            Self::Wire => "wire",
            Self::Json => "json",
            Self::Git => "git",
            Self::Board => "board",
            Self::Yaml => "yaml",
            Self::Hcl => "hcl",
            Self::Ebnf => "ebnf",
            Self::Regex => "regex",
            Self::Files => "files",
            Self::ChenEer => "chen_eer",
            Self::Chart => "chart",
            Self::Packet => "packet",
            Self::Sprites => "sprites",
            Self::Crash => "crash",
            Self::Unknown => "unknown",
        }
    }

    /// Determine the candidate diagram types from the source text by scanning
    /// for the first `@start<keyword>` (or `\start<keyword>`) directive.
    ///
    /// Returns an empty set when no valid directive is found.
    #[must_use]
    pub fn find_start_types(text: &str) -> HashSet<DiagramType> {
        let chars: Vec<char> = text.chars().collect();
        for i in 0..chars.len() {
            let c = chars[i];
            if c.is_whitespace() {
                continue;
            }
            if c != '@' && c != '\\' {
                return HashSet::new();
            }
            let pos = i + 1;
            if chars.len() - pos < 5 || !check("start", &chars, pos) {
                return HashSet::new();
            }
            let p = pos + 5;
            if p >= chars.len() {
                return HashSet::new();
            }
            return get_types(&chars, p);
        }
        HashSet::new()
    }
}

fn get_types(chars: &[char], p: usize) -> HashSet<DiagramType> {
    let mut set = HashSet::new();
    let c = chars[p].to_ascii_lowercase();
    match c {
        'b' => {
            if check("bpm", chars, p) {
                set.insert(DiagramType::Bpm);
            } else if check("board", chars, p) {
                set.insert(DiagramType::Board);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'c' => {
            if check("chart", chars, p) {
                set.insert(DiagramType::Chart);
            } else if check("creole", chars, p) {
                set.insert(DiagramType::Creole);
            } else if check("chronology", chars, p) {
                set.insert(DiagramType::Chronology);
            } else if check("chen", chars, p) {
                set.insert(DiagramType::ChenEer);
            } else if check("crash", chars, p) {
                set.insert(DiagramType::Crash);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'd' => {
            if check("dot", chars, p) {
                set.insert(DiagramType::Dot);
            } else if check("ditaa", chars, p) {
                set.insert(DiagramType::Ditaa);
            } else if check("def", chars, p) {
                set.insert(DiagramType::Definition);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'e' => {
            if check("ebnf", chars, p) {
                set.insert(DiagramType::Ebnf);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'f' => {
            if check("flow", chars, p) {
                set.insert(DiagramType::Flow);
            } else if check("files", chars, p) {
                set.insert(DiagramType::Files);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'g' => {
            if check("gantt", chars, p) {
                set.insert(DiagramType::Gantt);
            } else if check("git", chars, p) {
                set.insert(DiagramType::Git);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'h' => {
            if check("hcl", chars, p) {
                set.insert(DiagramType::Hcl);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'j' => {
            if check("jcckit", chars, p) {
                set.insert(DiagramType::Jcckit);
            } else if check("json", chars, p) {
                set.insert(DiagramType::Json);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'l' => {
            if check("latex", chars, p) {
                set.insert(DiagramType::Latex);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'm' => {
            if check("math", chars, p) {
                set.insert(DiagramType::Math);
            } else if check("mindmap", chars, p) {
                set.insert(DiagramType::Mindmap);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'n' => {
            if check("nwdiag", chars, p) {
                set.insert(DiagramType::Nwdiag);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'p' => {
            if check("project", chars, p) {
                set.insert(DiagramType::Gantt);
            } else if check("packetdiag", chars, p) {
                set.insert(DiagramType::Packet);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'r' => {
            if check("regex", chars, p) {
                set.insert(DiagramType::Regex);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        's' => {
            if check("salt", chars, p) {
                set.insert(DiagramType::Salt);
            } else if check("sprites", chars, p) {
                set.insert(DiagramType::Sprites);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'u' => {
            if check("uml", chars, p) {
                for t in [
                    DiagramType::Sequence,
                    DiagramType::State,
                    DiagramType::Class,
                    DiagramType::Object,
                    DiagramType::Activity,
                    DiagramType::Description,
                    DiagramType::Composite,
                    DiagramType::Timing,
                    DiagramType::Help,
                    DiagramType::Sprites,
                ] {
                    set.insert(t);
                }
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'w' => {
            if check("wire", chars, p) {
                set.insert(DiagramType::Wire);
            } else if check("wbs", chars, p) {
                set.insert(DiagramType::Wbs);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        'y' => {
            if check("yaml", chars, p) {
                set.insert(DiagramType::Yaml);
            } else {
                set.insert(DiagramType::Unknown);
            }
        }
        _ => {
            set.insert(DiagramType::Unknown);
        }
    }
    set
}

/// Case-insensitive prefix check of `key` against `chars` starting at `p`.
fn check(key: &str, chars: &[char], p: usize) -> bool {
    let len = key.len();
    if p + len > chars.len() {
        return false;
    }
    for (i, kc) in key.chars().enumerate() {
        let c = chars[p + i].to_ascii_lowercase();
        if c != kc {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_uml() {
        let types = DiagramType::find_start_types("@startuml\na->b\n@enduml");
        assert!(types.contains(&DiagramType::Sequence));
        assert!(types.contains(&DiagramType::Class));
        assert!(types.contains(&DiagramType::Activity));
        assert!(types.len() == 10);
    }

    #[test]
    fn find_gantt() {
        let types = DiagramType::find_start_types("@startgantt\n@endgantt");
        assert_eq!(types.len(), 1);
        assert!(types.contains(&DiagramType::Gantt));
    }

    #[test]
    fn find_mindmap() {
        let types = DiagramType::find_start_types("@startmindmap\n@endmindmap");
        assert_eq!(types.len(), 1);
        assert!(types.contains(&DiagramType::Mindmap));
    }

    #[test]
    fn find_unknown_keyword() {
        let types = DiagramType::find_start_types("@startxyz\n@endxyz");
        assert_eq!(types.len(), 1);
        assert!(types.contains(&DiagramType::Unknown));
    }

    #[test]
    fn no_start_directive() {
        let types = DiagramType::find_start_types("hello world");
        assert!(types.is_empty());
    }

    #[test]
    fn backslash_start() {
        let types = DiagramType::find_start_types("\\startuml\na->b\n\\enduml");
        assert!(types.contains(&DiagramType::Sequence));
    }

    #[test]
    fn human_readable() {
        assert_eq!(DiagramType::Sequence.human_readable_name(), "sequence");
        assert_eq!(DiagramType::Description.human_readable_name(), "component");
        assert_eq!(DiagramType::Class.human_readable_name(), "class");
    }

    #[test]
    fn legacy_uml() {
        assert!(DiagramType::Sequence.is_legacy_uml());
        assert!(DiagramType::Class.is_legacy_uml());
        assert!(!DiagramType::Gantt.is_legacy_uml());
        assert!(!DiagramType::Mindmap.is_legacy_uml());
    }
}
