//! Boundary lint — the ship-standard / ship-code architecture invariant.
//!
//! Ship standard (shipd with default features OFF) must carry ZERO
//! coding-class crates: `ship-code` and `ship-git` exist only behind the
//! `ship-code` capability. This is a standing architecture rule (the
//! physical seam), not a migration check — the standard dependency graph
//! is the contract that lets the capability be sold, mounted, and audited
//! as a separate surface.
//!
//! Mechanism: `cargo tree -p shipd --no-default-features` from the repo
//! root; any node naming `ship-code` or `ship-git` is a violation. The
//! grep matches package-name nodes (`ship-code vX`, `ship-git vX`), not
//! substrings of unrelated names.

use std::path::Path;
use std::process::Command;

pub fn run(repo_root: &Path) -> Result<(), String> {
    let out = Command::new("cargo")
        .args(["tree", "-p", "shipd", "--no-default-features", "--prefix", "none"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("cargo tree failed to spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo tree exited non-zero:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let tree = String::from_utf8_lossy(&out.stdout);
    let violations: Vec<&str> = tree
        .lines()
        .filter(|l| {
            let name = l.split_whitespace().next().unwrap_or("");
            name == "ship-code" || name == "ship-git"
        })
        .collect();
    if violations.is_empty() {
        println!("boundary clean: ship standard carries no ship-code/ship-git");
        Ok(())
    } else {
        Err(format!(
            "ship standard's dependency graph carries coding-class crates \
             (the standard/code seam is an architecture rule — these mount \
             only behind the `ship-code` capability):\n{}",
            violations.join("\n")
        ))
    }
}
