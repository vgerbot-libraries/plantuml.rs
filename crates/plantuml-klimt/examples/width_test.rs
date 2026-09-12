use plantuml_klimt::string_bounder_from_width_table::StringBounderFromWidthTable;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::file_format::FileFormat;
use plantuml_core::u_font::UFont;

fn main() {
    let bounder = StringBounderFromWidthTable::new(FileFormat::Svg);
    for (text, size) in [("Alice", 14), ("Bob", 14), ("hello", 13), ("hi", 13)] {
        let font = UFont::sans_serif(size);
        let dim = bounder.calculate_dimension(&font, text);
        println!("{} @{}: width={:.4}", text, size, dim.width());
    }
}
