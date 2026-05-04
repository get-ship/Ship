//! `cargo run -p sui-lint --example check -- <path>` — lint one file from CLI.

fn main() {
    let path_str = std::env::args().nth(1).expect("usage: check <path>");
    let path = std::path::Path::new(&path_str);
    let src = std::fs::read_to_string(path).expect("read");
    let findings = sui_lint::lint_controller_source(path, &src);
    println!("findings: {}", findings.len());
    for f in &findings {
        println!("  {}:{} [{}] {}", f.line, f.col, f.rule, f.message);
        if let Some(s) = &f.suggest {
            println!("    suggest: {}", s);
        }
    }
    std::process::exit(if findings.is_empty() { 0 } else { 1 });
}
