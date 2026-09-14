//! Standalone test that generates Rust SVGs for all 6 mindmap/WBS test cases.

use plantuml_mindmap::idea::{parse_mindmap_orgmode, parse_mindmap_plus};
use plantuml_mindmap::mindmap_renderer::render_mindmap_svg;
use plantuml_mindmap::wbs_element::parse_wbs_tree;
use plantuml_mindmap::wbs_renderer::render_wbs_svg;

#[test]
fn generate_all_svgs() {
    let mindmap_basic = vec!["* Root idea", "** First branch", "*** Sub idea", "*** Another sub idea", "** Second branch", "*** Detail"];
    let tree = parse_mindmap_orgmode(&mindmap_basic).unwrap();
    let svg = render_mindmap_svg(&tree);
    std::fs::write("/tmp/mindmap_basic_tree.rust.svg", &svg).unwrap();
    println!("mindmap_basic_tree: {} bytes", svg.len());

    let mindmap_plus = vec!["+ Root", "++ Child A", "++ Child B", "+++ Grandchild"];
    let tree = parse_mindmap_plus(&mindmap_plus).unwrap();
    let svg = render_mindmap_svg(&tree);
    std::fs::write("/tmp/mindmap_plus_syntax.rust.svg", &svg).unwrap();
    println!("mindmap_plus_syntax: {} bytes", svg.len());

    let mindmap_lr = vec!["+ Central topic", "-- Left idea", "--- Deeper left", "++ Right idea", "+++ Deeper right"];
    let tree = parse_mindmap_plus(&mindmap_lr).unwrap();
    let svg = render_mindmap_svg(&tree);
    std::fs::write("/tmp/mindmap_left_right_branches.rust.svg", &svg).unwrap();
    println!("mindmap_left_right_branches: {} bytes", svg.len());

    let wbs_basic = vec!["* Project", "** Phase 1", "*** Task A", "*** Task B", "** Phase 2", "*** Task C"];
    let tree = parse_wbs_tree(&wbs_basic).unwrap();
    let svg = render_wbs_svg(&tree);
    std::fs::write("/tmp/wbs_basic_tree.rust.svg", &svg).unwrap();
    println!("wbs_basic_tree: {} bytes", svg.len());

    let wbs_deep = vec!["* Product Launch", "** Planning", "*** Market research", "*** Budget planning", "** Execution", "*** Development", "**** Testing", "***** QA", "*** Marketing"];
    let tree = parse_wbs_tree(&wbs_deep).unwrap();
    let svg = render_wbs_svg(&tree);
    std::fs::write("/tmp/wbs_deeper_nesting.rust.svg", &svg).unwrap();
    println!("wbs_deeper_nesting: {} bytes", svg.len());

    let wbs_styled = vec!["* Project", "** Phase 1 [#LightBlue]", "*** Task A", "*** Task B", "** Phase 2", "*** Task C"];
    let tree = parse_wbs_tree(&wbs_styled).unwrap();
    let svg = render_wbs_svg(&tree);
    std::fs::write("/tmp/wbs_styled_nodes.rust.svg", &svg).unwrap();
    println!("wbs_styled_nodes: {} bytes", svg.len());

    println!("\nAll SVGs written to /tmp/");
}
