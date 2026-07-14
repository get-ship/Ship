use std::path::Path;

use anyhow::Context;

/// Compute a deterministic SHA-256 content hash for the stack tree at `root`.
///
/// The hash is the union of:
/// 1. The manifest file at the stack root (`stack.config.ts`).
/// 2. Every file under one of the domain dirs in
///    `stack_model::DOMAIN_DIRS` (the SAME allow-list pack walks).
///
/// Entries are sorted lexicographically by relative path. For each, the
/// accumulator is `"<rel-path>\0<byte-length>\0<sha256-of-content>"`. The
/// accumulator is itself SHA-256-hashed, and the result is returned as
/// `"sha256:<lowercase-hex>"`.
///
/// The hash and the pack allow-list MUST agree, or the content hash
/// recorded in `ship.json` would diverge from the bytes the install side
/// recomputes from the extracted tree (the `unpack_stack` re-hash
/// check).
pub fn compute_tree_hash(root: &Path) -> anyhow::Result<String> {
    let mut file_entries: Vec<(String, u64, String)> = Vec::new();

    let manifest_full = root.join(crate::archive::MANIFEST_PATH);
    if manifest_full.is_file() {
        // Cap the per-file read at MAX_BUNDLE_FILE — a single huge
        // manifest must not be slurped into memory.
        let content = read_file_capped(&manifest_full)?;
        file_entries.push((
            crate::archive::MANIFEST_PATH.to_string(),
            content.len() as u64,
            hex::encode_sha256(&content),
        ));
    }

    for domain in stack_model::DOMAIN_DIRS {
        let dir = root.join(domain);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir)
            .sort_by_file_name()
            .into_iter()
        {
            let entry = entry.with_context(|| {
                format!("walking package tree {}", dir.display())
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(root)
                .context("stripping root prefix")?;

            // Normalize to forward-slash separators for cross-platform
            // determinism.
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");

            // Cap the per-file read at MAX_BUNDLE_FILE — a single
            // multi-GB member must not be slurped into a single
            // allocation.
            let content = read_file_capped(entry.path())?;
            let byte_len = content.len() as u64;
            let file_hash = hex::encode_sha256(&content);
            file_entries.push((rel_str, byte_len, file_hash));
        }
    }

    // Lexicographic sort by relative path (walkdir sorts by filename
    // within each directory; re-sort globally to ensure full-path
    // ordering across the manifest + the union of domain dir walks).
    file_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut accumulator = String::new();
    for (rel_path, byte_len, file_hash) in &file_entries {
        accumulator.push_str(rel_path);
        accumulator.push('\0');
        accumulator.push_str(&byte_len.to_string());
        accumulator.push('\0');
        accumulator.push_str(file_hash);
    }

    Ok(format!("sha256:{}", hex::encode_sha256(accumulator.as_bytes())))
}

/// Compute SHA-256 hash of a single file.
/// Returns `"sha256:<lowercase-hex>"`.
pub fn compute_file_hash(path: &Path) -> anyhow::Result<String> {
    let content = read_file_capped(path)?;
    Ok(format!("sha256:{}", hex::encode_sha256(&content)))
}

/// Read a file's contents with the install pipeline's per-file ceiling
/// (a multi-GB member must not be read into one allocation). Centralised
/// so the cap is named once and the streaming helper is the only path
/// the install side uses for slurping.
fn read_file_capped(path: &Path) -> anyhow::Result<Vec<u8>> {
    let f = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    match bundle_limits::read_capped(f, bundle_limits::MAX_BUNDLE_FILE) {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            // Surface the bound name in the top-level error message so
            // the failure points at the policy, not the symptom.
            Err(anyhow::anyhow!(
                "reading {} exceeded MAX_BUNDLE_FILE ({} bytes): {}",
                path.display(),
                bundle_limits::MAX_BUNDLE_FILE,
                e
            ))
        }
    }
}

mod hex {
    use sha2::{Digest, Sha256};

    pub fn encode_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
#[path = "hash_tests.rs"]
mod tests;
