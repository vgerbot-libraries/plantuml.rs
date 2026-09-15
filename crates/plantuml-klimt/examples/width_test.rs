use plantuml_klimt::string_bounder_svg::StringBounderSvg;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::file_format::FileFormat;
use plantuml_core::u_font::UFont;

fn main() {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
    for (text, size) in [
        ("Alice", 14), ("Bob", 14), ("Foo", 14),
        ("Animal", 14), ("Dog", 14), ("Cat", 14),
        ("Shape", 14), ("Circle", 14), ("Square", 14),
        ("field: int", 14), ("privateField: String", 14),
        ("protectedField: bool", 14), ("method(): void", 14),
        ("helper(): int", 14),
    ] {
        let font_n = UFont::sans_serif(size);
        let font_i = UFont::sans_serif(size).with_style(plantuml_core::u_font::FontStyle::italic());
        let dim_n = bounder.calculate_dimension(&font_n, text);
        let dim_i = bounder.calculate_dimension(&font_i, text);
        println!("{} @{}: italic={:.6} normal={:.6}", text, size, dim_i.width(), dim_n.width());
    }
}
