//! Canonical set of stack-content domain directories.
//!
//! A stack's on-disk content lives under a small, fixed set of directories
//! beside the manifest. Every stack reader (pack, hash, dep resolver) asks
//! the same question — "is this entry under a domain dir?" — and gets the
//! same answer from this list. A file appearing in a domain dir IS the
//! registration; nothing else is stack content.
//!
//! The set is the SAME shape the resolver harvests from a member stack
//! (`ship_resolver::space_loader::PER_NOUN_SOURCE_DIRS`, plus the dirs the
//! resolver walks from `ExtensionLibrary`): agents, collections, pipelines,
//! triggers, views, skills, hooks. Naming the dirs here (rather than in
//! ship-store) keeps `stack-model` the leaf both crates already depend on,
//! so the rule lives in one place and ship-store stays TS-eval-free.

/// The conventional domain-dir names, relative to the stack root.
///
/// Order is meaningful for determinism in archive entries and human display;
/// pack walks this list and sorts the resulting entries lexicographically
/// before appending. The order in this list is the declaration order, not the
/// packed order — pack's sort is what governs the on-disk shape.
pub const DOMAIN_DIRS: &[&str] = &[
    "agents",
    "collections",
    "pipelines",
    "triggers",
    "views",
    "skills",
    "hooks",
];

/// True when `name` is a stack domain-dir basename (any depth).
pub fn is_domain_dir(name: &str) -> bool {
    DOMAIN_DIRS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_contains_canonical_seven() {
        // The seven dirs the rule guarantees. Adding or removing any
        // requires an explicit fence in the job spec — the set is the
        // truth the contract (pack, hash, dep-resolver) all read from.
        assert_eq!(DOMAIN_DIRS.len(), 7);
        for name in [
            "agents",
            "collections",
            "pipelines",
            "triggers",
            "views",
            "skills",
            "hooks",
        ] {
            assert!(DOMAIN_DIRS.contains(&name), "{name} missing from DOMAIN_DIRS");
        }
    }

    #[test]
    fn non_domain_dirs_are_rejected() {
        for name in [
            "target",
            "node_modules",
            "dist",
            "scripts",
            "ops",
            "contrib",
            "README.md",
            "justfile",
            ".git",
            "stack.config.ts",
        ] {
            assert!(
                !is_domain_dir(name),
                "{name} must not be a domain dir (it is build output, tooling, or the manifest itself)"
            );
        }
    }
}
