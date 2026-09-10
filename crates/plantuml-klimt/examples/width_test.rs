use plantuml_klimt::string_bounder_from_width_table::StringBounderFromWidthTable;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;
use plantuml_core::FileFormat;

fn main() {
    let bounder = StringBounderFromWidthTable::new(FileFormat::Svg);
    let font14 = UFont::sans_serif(14);
    let font13 = UFont::sans_serif(13);
    
    let alice_dim = bounder.calculate_dimension(&font14, "Alice");
    let bob_dim = bounder.calculate_dimension(&font14, "Bob");
    let hello_dim = bounder.calculate_dimension(&font13, "hello");
    
    println!("Alice width: {}", alice_dim.width());
    println!("Bob width: {}", bob_dim.width());
    println!("hello(13) width: {}", hello_dim.width());
    
    println!("Expected: Alice=30.45, Bob=24.938, hello=27.544");
}
