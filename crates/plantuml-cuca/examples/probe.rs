use plantuml_cuca::parse_entity_link_source;
fn main() {
    let src = [
        "interface \"User API\" as API",
        "[Client] ..> API : uses",
        "[Service] -- API : provides",
    ];
    let r = parse_entity_link_source(&src);
    for (n, e) in &r.entities {
        println!("ENT name={n} kind={:?} display={}", e.kind, e.display);
    }
    for l in &r.links {
        println!("LINK from={} to={} arrow={} label={:?}", l.from, l.to, l.arrow, l.label);
    }
}
