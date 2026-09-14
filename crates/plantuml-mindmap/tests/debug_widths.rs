//! Debug test that prints text widths for mindmap labels.

use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;
use plantuml_klimt::string_bounder_svg::StringBounderSvg;

#[test]
fn print_text_widths() {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(14);
    
    let labels = vec!["Root idea", "First branch", "Sub idea", "Another sub idea", "Second branch", "Detail"];
    for label in &labels {
        let dim = bounder.calculate_dimension(&font, label);
        println!("{}: width={:.10}, height={:.10}", label, dim.width(), dim.height());
    }
    
    // Also check WBS font size
    let font12 = UFont::sans_serif(12);
    let wbs_labels = vec!["Project", "Phase 1", "Task A", "Task B", "Phase 2", "Task C"];
    for label in &wbs_labels {
        let dim = bounder.calculate_dimension(&font12, label);
        println!("{}: width={:.10}, height={:.10}", label, dim.width(), dim.height());
    }
}
