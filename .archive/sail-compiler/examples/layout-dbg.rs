use sail_compiler::parse;
fn main() {
    let s = "<template><extends src=\"layout.set\" /><fill name=\"content\"><p>HELLO</p></fill></template>";
    let f = parse::parse(s).unwrap();
    let t = f.template.unwrap();
    println!("extends: {:?}", t.extends);
    println!("root: {:#?}", t.root);
}
