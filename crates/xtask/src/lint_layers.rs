//! Lint the substrate / platform layer boundary.
//!
//! Two rules, mechanically enforced from `Cargo.toml` of each layer:
//!
//! 1. `ship-substrate` declares no dependency on `ship-platform`.
//! 2. Neither layer lists itself.
//!
//! Catches the most common form of layer regression — a one-line dep
//! addition "for convenience" that wires the wrong direction. Run via
//! `cargo xtask lint-layers`; CI fails the build on any violation.

use std::fs;
use std::path::Path;

#[derive(Debug)]
struct Layer {
    name: &'static str,
    manifest: &'static str,
    forbidden: &'static [&'static str],
}

const LAYERS: &[Layer] = &[
    Layer {
        name: "ship-substrate",
        manifest: "crates/core/ship-substrate/Cargo.toml",
        forbidden: &["ship-platform"],
    },
    Layer {
        name: "ship-platform",
        manifest: "crates/core/ship-platform/Cargo.toml",
        // No forbidden upstream — platform is the top of the core stack.
        forbidden: &[],
    },
];

pub fn run(repo_root: &Path) -> Result<(), String> {
    let mut errors = Vec::new();
    for layer in LAYERS {
        let path = repo_root.join(layer.manifest);
        let toml = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        for forbidden in layer.forbidden {
            // Match any dep table entry where the key is the forbidden crate
            // name. A line like `ship-platform = { path = "..." }` or
            // `ship-platform = "0.1"` both start with the name followed by
            // optional whitespace + `=`.
            for (lineno, line) in toml.lines().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    continue;
                }
                if let Some(rest) = trimmed.strip_prefix(forbidden)
                    && rest.trim_start().starts_with('=')
                {
                    errors.push(format!(
                        "{} (line {}): forbidden layer dep `{}` — {} must not depend on {}",
                        layer.manifest,
                        lineno + 1,
                        forbidden,
                        layer.name,
                        forbidden,
                    ));
                }
            }
        }
        // Self-reference check.
        let self_pat = format!("{} = ", layer.name);
        if toml.lines().any(|l| {
            let t = l.trim_start();
            !t.starts_with('#') && t.starts_with(&self_pat)
        }) {
            errors.push(format!(
                "{}: declares itself as a dependency",
                layer.manifest
            ));
        }
    }

    if errors.is_empty() {
        println!("layer boundaries clean ({} layers checked)", LAYERS.len());
        return Ok(());
    }
    for e in &errors {
        eprintln!("  {e}");
    }
    Err(format!("layer boundary violations: {}", errors.len()))
}
