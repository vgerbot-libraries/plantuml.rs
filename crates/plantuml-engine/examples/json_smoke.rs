use plantuml_engine::render;
use plantuml_core::FileFormat;

fn main() {
    // JSON smoke test
    let json_src = r#"@startjson
{
  "name": "Alice",
  "age": 30,
  "active": true,
  "address": {
    "city": "NYC",
    "zip": "10001"
  },
  "hobbies": ["reading", "coding"]
}
@endjson"#;
    
    let result = render(json_src, FileFormat::Svg);
    match &result {
        Ok(svg) => {
            println!("JSON SVG length: {}", svg.len());
            println!("Contains <svg>: {}", svg.contains("<svg"));
            println!("Contains </svg>: {}", svg.contains("</svg>"));
            println!("Contains 'name': {}", svg.contains("name"));
            println!("Contains 'Alice': {}", svg.contains("Alice"));
            println!("Contains 'address': {}", svg.contains("address"));
            println!("Contains 'hobbies': {}", svg.contains("hobbies"));
            // Print first 500 chars
            println!("\n--- First 500 chars ---");
            println!("{}", &svg[..svg.len().min(500)]);
        }
        Err(e) => println!("JSON ERROR: {:?}", e),
    }
    
    println!("\n---\n");
    
    // YAML smoke test
    let yaml_src = r#"@startyaml
name: Alice
age: 30
active: true
address:
  city: NYC
  zip: "10001"
hobbies:
  - reading
  - coding
@endyaml"#;
    
    let result = render(yaml_src, FileFormat::Svg);
    match &result {
        Ok(svg) => {
            println!("YAML SVG length: {}", svg.len());
            println!("Contains <svg>: {}", svg.contains("<svg"));
            println!("Contains 'name': {}", svg.contains("name"));
            println!("Contains 'Alice': {}", svg.contains("Alice"));
        }
        Err(e) => println!("YAML ERROR: {:?}", e),
    }
    
    // Invalid JSON test
    let bad_json = "@startjson\n{invalid json}\n@endjson";
    let result = render(bad_json, FileFormat::Svg);
    match &result {
        Ok(svg) => {
            println!("\nInvalid JSON SVG (first 400 chars):");
            println!("{}", &svg[..svg.len().min(400)]);
            println!("\nContains 'does not sound': {}", svg.contains("does\u{00a0}not\u{00a0}sound"));
            println!("Contains 'sound': {}", svg.contains("sound"));
        }
        Err(e) => println!("Invalid JSON error: {:?}", e),
    }
}

