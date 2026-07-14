//! ship.lock parser and writer.
//!
//! `ship.lock` is the registry dependency lockfile, at the STACK ROOT beside `stack.config.ts` (lockfile-beside-manifest; ruled 2026-07-12).
//! It records direct resolved dependencies — path, version, commit, and content hash.
//! On disk, the file is JSONC (matches every other `.ship/` config: agents,
//! skills, views, pipelines, collections). The writer emits pure JSON for
//! deterministic diffs; the reader accepts comments and trailing commas.
//!
//! Format versions:
//! - **v1** (pre-rails): every entry is implicitly a stack package. v1 files
//!   still parse cleanly under v2 — the missing `rails` field defaults to
//!   [`Rails::Stack`], matching the v1 assumption.
//! - **v2** (additive): each entry carries a [`Rails`] tag (`"stack"` or
//!   `"npm"`). Writers emit `version: 2`; the default for a freshly
//!   constructed `ShipLock` is v2. The two rails coexist in one lockfile:
//!   stack packages go through the registry resolver, npm packages through
//!   the component rail (wave 2 — see report-registry-shapes.md Q4).
//!
//! NOTE: `.ship/ship.state` (formerly `.ship/ship.lock`) stores workspace state
//! (active_profile, compiled_at). These are completely separate files.

use std::path::Path;

use anyhow::Context;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::manifest::{ManifestDepValue, ShipManifest};

// ── Rails ──────────────────────────────────────────────────────────────────────

/// Which rail a locked package rides. The lockfile carries both rails so the
/// user sees one file; the per-entry `rails` field tells install where to send
/// each package (registry resolver vs. the component rail).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Rails {
    /// A stack package — goes through the stack registry (the canonical rail;
    /// the v1 default for entries that pre-date the field).
    #[default]
    Stack,
    /// A component package — goes through npm (the component rail).
    Npm,
}

impl Rails {
    /// Wire form: lowercase string. Stable on disk.
    pub fn as_str(self) -> &'static str {
        match self {
            Rails::Stack => "stack",
            Rails::Npm => "npm",
        }
    }

    /// Classify a manifest dependency key into a rail. Pure function of the
    /// key — survives alias-resolution round-trips (registry-shapes Q4):
    /// - `@scope/...` (npm package name) → `Npm`, **except** the exact
    ///   two-segment form `@scope/stack` which the operator reserved for
    ///   stack packages addressed through the npm-shaped edge of the
    ///   registry. `@scope/sub/stack` is the npm rail (npm's name grammar
    ///   forbids multi-segment names today, but the rule stays one-sided:
    ///   only `@scope/stack` exactly is the stack edge form).
    /// - `host.tld/owner/repo` (e.g. `github.com/o/r`) → `Stack`.
    /// - Anything else → `Stack` (canonical default; matches the v1
    ///   assumption that every entry is a stack package).
    ///
    /// The classifier runs at parse time, not resolve time: the manifest
    /// tells the lockfile the rail, not the resolver.
    pub fn classify(dep_key: &str) -> Rails {
        let trimmed = dep_key.trim();
        if trimmed.is_empty() {
            return Rails::Stack;
        }
        if let Some(rest) = trimmed.strip_prefix('@') {
            // `rest` is `scope/...`. Split on `/` and inspect the shape:
            // - exactly two segments and the second is `stack` → Stack
            //   (the explicit stack-on-npm edge form).
            // - any other `@scope/...` shape → Npm.
            let mut parts = rest.split('/');
            let scope = parts.next().unwrap_or("");
            let tail = parts.next().unwrap_or("");
            let has_more = parts.next().is_some();
            if !scope.is_empty() && tail == "stack" && !has_more {
                return Rails::Stack;
            }
            return Rails::Npm;
        }
        // `host.tld/owner/repo` — the canonical stack key. Anything that
        // doesn't look npm-shaped falls into the stack rail (v1 default).
        Rails::Stack
    }
}

impl Serialize for Rails {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Rails {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "stack" => Ok(Rails::Stack),
            "npm" => Ok(Rails::Npm),
            other => Err(serde::de::Error::custom(format!(
                "unknown rails tag '{other}'; expected \"stack\" or \"npm\""
            ))),
        }
    }
}

impl std::fmt::Display for Rails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Rails {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stack" => Ok(Rails::Stack),
            "npm" => Ok(Rails::Npm),
            other => Err(format!(
                "unknown rails tag '{other}'; expected \"stack\" or \"npm\""
            )),
        }
    }
}

// ── Types ──────────────────────────────────────────────────────────────────────

/// A single locked package entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LockPackage {
    /// Package path, e.g. `github.com/owner/repo`.
    pub path: String,
    /// Resolved version string (tag or branch@commit).
    pub version: String,
    /// Exact 40-char hex commit SHA.
    pub commit: String,
    /// Content hash in `sha256:<hex>` format (canonical tree hash).
    pub hash: String,
    /// Which rail this entry rides. Defaults to [`Rails::Stack`] for entries
    /// written under v1 (the field is absent in v1 lockfiles).
    #[serde(default)]
    pub rails: Rails,
}

/// The parsed `ship.lock` file (stack root).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShipLock {
    /// Lockfile format version. Writers emit `2`; the reader accepts `1`
    /// (defaults `rails` to [`Rails::Stack`] on every entry) and `2`.
    pub version: u32,
    /// Locked packages, one entry per direct dependency.
    #[serde(default)]
    pub packages: Vec<LockPackage>,
}

impl Default for ShipLock {
    fn default() -> Self {
        ShipLock {
            version: 2,
            packages: vec![],
        }
    }
}

// ── impl ShipLock ──────────────────────────────────────────────────────────────

impl ShipLock {
    /// Parse a `ship.lock` file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        Self::from_jsonc_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Parse a `ship.lock` JSONC string. Accepts `//` line comments,
    /// `/* */` block comments, and trailing commas — pure JSON also parses.
    pub fn from_jsonc_str(s: &str) -> anyhow::Result<Self> {
        serde_json::from_str(&strip_jsonc(s)).map_err(Into::into)
    }
    /// Serialize to a deterministic JSON string.
    ///
    /// Packages are sorted by path so the output is byte-identical for the
    /// same logical content. The writer always emits `version: 2` — v1
    /// in-memory values are upgraded in place, the per-entry `rails` field
    /// rides forward on the entries that had no value (defaulted to Stack
    /// on read). This is the additive cutover: a v1 file written through
    /// `to_json_string` becomes a v2 file.
    pub fn to_json_string(&self) -> anyhow::Result<String> {
        let mut sorted = self.clone();
        sorted.packages.sort_by(|a, b| a.path.cmp(&b.path));
        sorted.version = 2;
        let mut s = serde_json::to_string_pretty(&sorted)?;
        s.push('\n');
        Ok(s)
    }

    /// Atomically write the lockfile: write to `<path>.tmp` then rename.
    pub fn write_atomic(&self, path: &Path) -> anyhow::Result<()> {
        let tmp = path.with_extension("lock.tmp");
        let content = self.to_json_string()?;
        std::fs::write(&tmp, &content)
            .with_context(|| format!("Cannot write temp lockfile {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Cannot rename lockfile into place at {}", path.display()))?;
        Ok(())
    }

    /// Compare manifest dependencies against locked packages.
    ///
    /// Returns `(added, removed)` where:
    /// - `added`   = dep paths in manifest but not in lockfile
    /// - `removed` = dep paths in lockfile but not in manifest
    pub fn check_sync(&self, manifest: &ShipManifest) -> (Vec<String>, Vec<String>) {
        let manifest_keys: std::collections::HashSet<&str> =
            manifest.dependencies.keys().map(|k| k.as_str()).collect();
        let lock_paths: std::collections::HashSet<&str> =
            self.packages.iter().map(|p| p.path.as_str()).collect();

        let added: Vec<String> = manifest_keys
            .difference(&lock_paths)
            .map(|s| s.to_string())
            .collect();
        let removed: Vec<String> = lock_paths
            .difference(&manifest_keys)
            .map(|s| s.to_string())
            .collect();
        (added, removed)
    }
}

// ── Manifest → rail classifier ────────────────────────────────────────────────
//
// Pure function of a manifest's `dependencies` map: split it into the two
// groups the install flow routes separately. The classifier runs at parse
// time, before any resolution work, so the lockfile writer can tag every
// entry with its rail in one pass. Wave 2 (install rewrite) consumes the
// split groups — this job only owns the classification, not the install
// routing.

/// A manifest's dependencies grouped by the rail they ride.
///
/// Iteration order matches the manifest's order (`IndexMap` preserves
/// insertion). `stack` and `npm` are disjoint and together cover every dep
/// key the manifest declares.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClassifiedDeps {
    /// Stack-rail deps — registry resolver. `host.tld/owner/repo` keys, plus
    /// the explicit `@scope/stack` form.
    pub stack: Vec<(String, ManifestDepValue)>,
    /// Component-rail deps — npm. `@scope/...` keys (other than `@scope/stack`).
    pub npm: Vec<(String, ManifestDepValue)>,
}

impl ClassifiedDeps {
    /// Total number of deps across both rails.
    pub fn total(&self) -> usize {
        self.stack.len() + self.npm.len()
    }

    /// True when no deps were declared.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty() && self.npm.is_empty()
    }
}

/// Split a manifest's `dependencies` map by rail. The order of each group
/// matches the manifest's declaration order; the union of `stack` and `npm`
/// is exactly the manifest's dep set.
///
/// Classification rule (per registry-shapes Q4, see `Rails::classify`):
/// - `@scope/...` → npm (component rail), **except** `@scope/stack` → stack.
/// - `host.tld/owner/repo` and everything else → stack.
pub fn classify_manifest_deps(manifest: &ShipManifest) -> ClassifiedDeps {
    let mut out = ClassifiedDeps::default();
    for (key, value) in &manifest.dependencies {
        match Rails::classify(key) {
            Rails::Stack => out.stack.push((key.clone(), value.clone())),
            Rails::Npm => out.npm.push((key.clone(), value.clone())),
        }
    }
    out
}

/// Build a fresh v2 `ShipLock` from a manifest, tagging each entry with its
/// rail. The caller supplies the resolved package records (the classifier
/// only knows which rail each dep rides; resolution is wave 2).
///
/// `resolved` keys must match the manifest's dep keys. Missing keys are an
/// error — every declared dep must have a resolution record. Extra keys
/// (resolutions for deps the manifest doesn't declare) are likewise an
/// error: the lockfile carries the manifest's deps, nothing more.
pub fn build_lock_from_manifest(
    manifest: &ShipManifest,
    mut resolved: IndexMap<String, LockPackage>,
) -> anyhow::Result<ShipLock> {
    let mut packages = Vec::with_capacity(resolved.len());
    for (key, _dep) in &manifest.dependencies {
        let pkg = resolved.shift_remove(key).ok_or_else(|| {
            anyhow::anyhow!("manifest declares dep '{key}' but no resolution record was provided")
        })?;
        // Manifest is the source of truth for the rail — the resolver doesn't
        // get to override it. The classifier is a pure function of the key
        // and survives alias resolution, so this matches the on-disk intent.
        let expected = Rails::classify(key);
        if pkg.rails != expected {
            anyhow::bail!(
                "dep '{key}' resolution record carries rails '{}' but the classifier says '{}'",
                pkg.rails,
                expected
            );
        }
        packages.push(pkg);
    }
    if !resolved.is_empty() {
        let extra: Vec<&str> = resolved.keys().map(String::as_str).collect();
        anyhow::bail!("resolution records include deps the manifest does not declare: {extra:?}");
    }
    packages.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ShipLock {
        version: 2,
        packages,
    })
}

// ── JSONC helper ──────────────────────────────────────────────────────────────
//
// Strip `//` line comments, `/* */` block comments, and trailing commas so
// the result parses as plain JSON. Respects string contents so `//` inside
// `"http://x"` is preserved. Kept local — the compiler crate is WASM-safe
// and `ship-substrate::fs_util` is not in scope.

fn strip_jsonc(input: &str) -> String {
    // Phase 1: strip comments while respecting string literals.
    let mut stage = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            stage.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            stage.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\n' {
                            stage.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        stage.push(c);
    }
    // Phase 2: strip trailing commas before `]` / `}`. Walks bytes for
    // cheap lookahead; non-ASCII passes through as-is.
    let src = stage.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < src.len() {
        let b = src[i];
        if in_str {
            out.push(b);
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            let mut j = i + 1;
            while j < src.len() && (src[j] as char).is_whitespace() {
                j += 1;
            }
            if j < src.len() && (src[j] == b']' || src[j] == b'}') {
                i = j;
                continue;
            }
        }
        out.push(b);
        i += 1;
    }
    // Safety: input was valid UTF-8; we only removed ASCII bytes outside
    // string contexts, never split a multi-byte sequence.
    String::from_utf8(out).unwrap_or(stage)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ShipManifest;

    fn sample_lock() -> ShipLock {
        ShipLock {
            version: 2,
            packages: vec![
                LockPackage {
                    path: "github.com/b/second".into(),
                    version: "1.0.0".into(),
                    commit: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                    hash: "sha256:bbbb".into(),
                    rails: Rails::Stack,
                },
                LockPackage {
                    path: "github.com/a/first".into(),
                    version: "2.0.0".into(),
                    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    hash: "sha256:aaaa".into(),
                    rails: Rails::Stack,
                },
            ],
        }
    }

    fn minimal_manifest_jsonc() -> &'static str {
        r#"{
            "module": { "name": "github.com/x/y", "version": "1.0.0" }
        }"#
    }

    #[test]
    fn to_json_string_is_sorted() {
        let lock = sample_lock();
        let s = lock.to_json_string().unwrap();
        let a_pos = s.find("github.com/a/first").unwrap();
        let b_pos = s.find("github.com/b/second").unwrap();
        assert!(a_pos < b_pos, "packages should be sorted by path");
    }

    #[test]
    fn to_json_string_deterministic() {
        let lock = sample_lock();
        assert_eq!(
            lock.to_json_string().unwrap(),
            lock.to_json_string().unwrap()
        );
    }

    #[test]
    fn round_trip_via_file() {
        let lock = sample_lock();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        lock.write_atomic(&path).unwrap();
        let loaded = ShipLock::from_file(&path).unwrap();
        assert_eq!(loaded.version, 2);
        let mut names: Vec<_> = loaded.packages.iter().map(|p| p.path.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["github.com/a/first", "github.com/b/second"]);
    }

    #[test]
    fn from_file_missing_returns_error() {
        let r = ShipLock::from_file(std::path::Path::new("/nonexistent/ship.lock"));
        assert!(r.is_err());
    }

    #[test]
    fn check_sync_both_empty() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_jsonc_str(minimal_manifest_jsonc()).unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_added_dep() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/x/y", "version": "1.0.0" },
                "dependencies": { "github.com/a/b": "^1.0.0" }
            }"#,
        )
        .unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert_eq!(added, vec!["github.com/a/b"]);
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_removed_dep() {
        let lock = ShipLock {
            version: 2,
            packages: vec![LockPackage {
                path: "github.com/a/b".into(),
                version: "1.0.0".into(),
                commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                hash: "sha256:aaaa".into(),
                rails: Rails::Stack,
            }],
        };
        let manifest = ShipManifest::from_jsonc_str(minimal_manifest_jsonc()).unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert_eq!(removed, vec!["github.com/a/b"]);
    }

    #[test]
    fn write_atomic_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        assert!(!path.exists());
        ShipLock::default().write_atomic(&path).unwrap();
        assert!(path.exists());
        assert!(!dir.path().join("ship.lock.tmp").exists());
    }

    #[test]
    fn version_field_is_written() {
        let s = ShipLock::default().to_json_string().unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v.get("version").and_then(|x| x.as_u64()), Some(2));
    }

    #[test]
    fn jsonc_comments_and_trailing_commas_parse() {
        let src = r#"{
            // top-level comment
            "version": 1,
            "packages": [
                {
                    "path": "github.com/owner/repo",
                    "version": "main",
                    "commit": "abcdef0123456789abcdef0123456789abcdef01",
                    "hash": "sha256:deadbeef", /* trailing comma below is JSONC-only */
                },
            ],
        }"#;
        let lock = ShipLock::from_jsonc_str(src).expect("JSONC must parse");
        assert_eq!(lock.version, 1);
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].path, "github.com/owner/repo");
    }

    #[test]
    fn comment_inside_string_value_preserved() {
        // The `//` inside the URL must not be treated as a comment.
        let src = r#"{
            "version": 1,
            "packages": [{"path": "github.com/a/b", "version": "https://example.com/x", "commit": "0123456789abcdef0123456789abcdef01234567", "hash": "sha256:x"}]
        }"#;
        let lock = ShipLock::from_jsonc_str(src).expect("URL-with-slashes must parse");
        assert_eq!(lock.packages[0].version, "https://example.com/x");
    }

    #[test]
    fn rails_default_is_stack() {
        // v1 lockfiles have no `rails` field; on read it must default to Stack.
        let v1_src = r#"{
            "version": 1,
            "packages": [{
                "path": "github.com/o/r",
                "version": "1.0.0",
                "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "hash": "sha256:aaaa"
            }]
        }"#;
        let lock = ShipLock::from_jsonc_str(v1_src).expect("v1 must parse as v2");
        assert_eq!(lock.version, 1, "version field is preserved on read");
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(
            lock.packages[0].rails,
            Rails::Stack,
            "missing rails defaults to Stack (v1 assumption)"
        );
    }

    #[test]
    fn v1_lockfile_round_trips_via_writer_with_rails_inferred() {
        // Read a v1 file (no `rails` field), write it back: the writer must
        // emit the v2 schema with the inferred rails. Re-reading must give
        // the same logical content.
        let v1_src = r#"{
            "version": 1,
            "packages": [
                {
                    "path": "github.com/a/first",
                    "version": "1.0.0",
                    "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "hash": "sha256:aaaa"
                }
            ]
        }"#;
        let lock = ShipLock::from_jsonc_str(v1_src).unwrap();
        let s = lock.to_json_string().unwrap();
        // The writer bumped the lockfile to v2.
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v.get("version").and_then(|x| x.as_u64()), Some(2));
        // The inferred rail rode the entry forward.
        let pkg = &v.get("packages").unwrap().as_array().unwrap()[0];
        assert_eq!(
            pkg.get("rails").and_then(|x| x.as_str()),
            Some("stack"),
            "writer emits the inferred rail as 'stack'"
        );
    }

    #[test]
    fn v2_lockfile_round_trips_with_rails_per_entry() {
        let lock = ShipLock {
            version: 2,
            packages: vec![
                LockPackage {
                    path: "github.com/o/r".into(),
                    version: "1.0.0".into(),
                    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    hash: "sha256:aaaa".into(),
                    rails: Rails::Stack,
                },
                LockPackage {
                    path: "@ship/sui-runtime".into(),
                    version: "1.2.3".into(),
                    commit: String::new(),
                    hash: "sha512:bbb".into(),
                    rails: Rails::Npm,
                },
            ],
        };
        let s = lock.to_json_string().unwrap();
        let loaded = ShipLock::from_jsonc_str(&s).unwrap();
        assert_eq!(loaded.version, 2);
        assert_eq!(loaded.packages.len(), 2);
        // Find each entry by path (writer sorts alphabetically).
        let by_path: std::collections::HashMap<&str, &LockPackage> = loaded
            .packages
            .iter()
            .map(|p| (p.path.as_str(), p))
            .collect();
        assert_eq!(by_path["github.com/o/r"].rails, Rails::Stack);
        assert_eq!(by_path["@ship/sui-runtime"].rails, Rails::Npm);
    }

    #[test]
    fn rails_serializes_as_lowercase_string() {
        let lock = ShipLock {
            version: 2,
            packages: vec![LockPackage {
                path: "github.com/o/r".into(),
                version: "1.0.0".into(),
                commit: "a".repeat(40),
                hash: "sha256:a".into(),
                rails: Rails::Npm,
            }],
        };
        let s = lock.to_json_string().unwrap();
        assert!(
            s.contains("\"rails\": \"npm\""),
            "Npm serializes as the lowercase string \"npm\"; got: {s}"
        );
    }

    #[test]
    fn rails_rejects_unknown_string() {
        let src = r#"{
            "version": 2,
            "packages": [{
                "path": "github.com/o/r",
                "version": "1.0.0",
                "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "hash": "sha256:a",
                "rails": "cargo"
            }]
        }"#;
        let err = ShipLock::from_jsonc_str(src).expect_err("unknown rail must error");
        let msg = err.to_string();
        assert!(
            msg.contains("cargo") && msg.contains("stack") && msg.contains("npm"),
            "error must name the bad value and the allowed set; got: {msg}"
        );
    }

    #[test]
    fn rails_from_str_round_trip() {
        for rail in [Rails::Stack, Rails::Npm] {
            let s = rail.to_string();
            let parsed: Rails = s.parse().unwrap();
            assert_eq!(parsed, rail);
            assert_eq!(rail.as_str(), s.as_str());
        }
    }

    // ── Classifier ──────────────────────────────────────────────────────────

    #[test]
    fn classify_host_tenant_repo_is_stack() {
        assert_eq!(Rails::classify("github.com/o/r"), Rails::Stack);
        assert_eq!(Rails::classify("gitlab.com/o/r"), Rails::Stack);
        assert_eq!(Rails::classify("example.org/o/r"), Rails::Stack);
    }

    #[test]
    fn classify_at_scope_is_npm() {
        assert_eq!(Rails::classify("@ship/sui-runtime"), Rails::Npm);
        assert_eq!(Rails::classify("@ship-cloud/anything"), Rails::Npm);
        assert_eq!(Rails::classify("@scope/lib"), Rails::Npm);
    }

    #[test]
    fn classify_at_scope_stack_is_stack() {
        // `@scope/stack` is the explicit stack-on-npm edge form.
        assert_eq!(Rails::classify("@ship/stack"), Rails::Stack);
        assert_eq!(Rails::classify("@anything/stack"), Rails::Stack);
    }

    #[test]
    fn classify_at_scope_with_internal_slashes_is_npm() {
        // `@scope/sub/path` — the `stack` tail is not the last segment, so
        // this is the npm rail (real npm packages don't have multi-segment
        // names, but the rule stays one-sided: only exact `@.../stack` is
        // the stack edge form).
        assert_eq!(Rails::classify("@scope/sub/stack"), Rails::Npm);
    }

    #[test]
    fn classify_empty_is_stack() {
        // Empty / whitespace is the canonical default (Stack). The manifest
        // validator rejects empty dep names long before we get here, so
        // this is purely a defensive default.
        assert_eq!(Rails::classify(""), Rails::Stack);
        assert_eq!(Rails::classify("   "), Rails::Stack);
    }

    #[test]
    fn classify_manifest_deps_splits_two_rails() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" },
                "dependencies": {
                    "github.com/a/host-pkg": "^1.0.0",
                    "@ship/sui-runtime": "^1.2.0",
                    "@ship/stack": "0.1.0",
                    "@ship-cloud/other": "2.0.0"
                }
            }"#,
        )
        .unwrap();
        let classified = classify_manifest_deps(&manifest);
        assert_eq!(classified.total(), 4);
        // Order in each group matches the manifest's declaration order.
        assert_eq!(
            classified
                .stack
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>(),
            vec!["github.com/a/host-pkg", "@ship/stack"],
        );
        assert_eq!(
            classified
                .npm
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>(),
            vec!["@ship/sui-runtime", "@ship-cloud/other"],
        );
    }

    #[test]
    fn classify_manifest_deps_empty_manifest() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" }
            }"#,
        )
        .unwrap();
        let classified = classify_manifest_deps(&manifest);
        assert!(classified.is_empty());
        assert_eq!(classified.total(), 0);
    }

    // ── build_lock_from_manifest ───────────────────────────────────────────

    fn pkg(path: &str, version: &str, commit: &str, hash: &str, rails: Rails) -> LockPackage {
        LockPackage {
            path: path.into(),
            version: version.into(),
            commit: commit.into(),
            hash: hash.into(),
            rails,
        }
    }

    #[test]
    fn build_lock_from_manifest_tags_rails_and_writes_v2() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" },
                "dependencies": {
                    "github.com/a/host-pkg": "^1.0.0",
                    "@ship/sui-runtime": "^1.2.0"
                }
            }"#,
        )
        .unwrap();
        let mut resolved: IndexMap<String, LockPackage> = IndexMap::new();
        resolved.insert(
            "github.com/a/host-pkg".into(),
            pkg(
                "github.com/a/host-pkg",
                "1.0.0",
                &"a".repeat(40),
                "sha256:ha",
                Rails::Stack,
            ),
        );
        resolved.insert(
            "@ship/sui-runtime".into(),
            pkg("@ship/sui-runtime", "1.2.3", "", "sha512:hb", Rails::Npm),
        );
        let lock = build_lock_from_manifest(&manifest, resolved).unwrap();
        assert_eq!(lock.version, 2, "writer path emits v2");
        assert_eq!(lock.packages.len(), 2);
        // Packages are sorted by path; @ship sorts before github.com.
        assert_eq!(lock.packages[0].path, "@ship/sui-runtime");
        assert_eq!(lock.packages[0].rails, Rails::Npm);
        assert_eq!(lock.packages[1].path, "github.com/a/host-pkg");
        assert_eq!(lock.packages[1].rails, Rails::Stack);
    }

    #[test]
    fn build_lock_from_manifest_rejects_rails_mismatch() {
        // The resolver tagged `@ship/sui-runtime` as Stack, but the
        // classifier (driven by the dep key) says Npm. Manifest wins.
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" },
                "dependencies": { "@ship/sui-runtime": "^1.2.0" }
            }"#,
        )
        .unwrap();
        let mut resolved: IndexMap<String, LockPackage> = IndexMap::new();
        resolved.insert(
            "@ship/sui-runtime".into(),
            pkg(
                "@ship/sui-runtime",
                "1.2.3",
                "",
                "sha512:hb",
                Rails::Stack, // wrong
            ),
        );
        let err = build_lock_from_manifest(&manifest, resolved).expect_err("must reject");
        let msg = err.to_string();
        assert!(msg.contains("@ship/sui-runtime"), "names the dep: {msg}");
        assert!(msg.contains("stack"), "mentions the bad tag: {msg}");
        assert!(msg.contains("npm"), "mentions the expected tag: {msg}");
    }

    #[test]
    fn build_lock_from_manifest_rejects_missing_resolution() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" },
                "dependencies": { "github.com/a/pkg": "^1.0.0" }
            }"#,
        )
        .unwrap();
        let err = build_lock_from_manifest(&manifest, IndexMap::new())
            .expect_err("missing resolution must error");
        assert!(err.to_string().contains("github.com/a/pkg"));
    }

    #[test]
    fn build_lock_from_manifest_rejects_extra_resolution() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" }
            }"#,
        )
        .unwrap();
        let mut resolved: IndexMap<String, LockPackage> = IndexMap::new();
        resolved.insert(
            "github.com/a/pkg".into(),
            pkg(
                "github.com/a/pkg",
                "1.0.0",
                &"a".repeat(40),
                "sha256:a",
                Rails::Stack,
            ),
        );
        let err = build_lock_from_manifest(&manifest, resolved).expect_err("extra must error");
        assert!(err.to_string().contains("github.com/a/pkg"));
    }

    #[test]
    fn build_lock_from_manifest_round_trips_via_file() {
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/o/r", "version": "1.0.0" },
                "dependencies": {
                    "github.com/a/host": "^1.0.0",
                    "@ship/sui-runtime": "^1.2.0"
                }
            }"#,
        )
        .unwrap();
        let mut resolved: IndexMap<String, LockPackage> = IndexMap::new();
        resolved.insert(
            "github.com/a/host".into(),
            pkg(
                "github.com/a/host",
                "1.0.0",
                &"a".repeat(40),
                "sha256:ha",
                Rails::Stack,
            ),
        );
        resolved.insert(
            "@ship/sui-runtime".into(),
            pkg("@ship/sui-runtime", "1.2.3", "", "sha512:hb", Rails::Npm),
        );
        let lock = build_lock_from_manifest(&manifest, resolved).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        lock.write_atomic(&path).unwrap();
        let loaded = ShipLock::from_file(&path).unwrap();
        assert_eq!(loaded.version, 2);
        assert_eq!(loaded.packages.len(), 2);
        // Round-trip preserved the per-entry rail.
        let npm_pkg = loaded
            .packages
            .iter()
            .find(|p| p.path == "@ship/sui-runtime")
            .unwrap();
        let stack_pkg = loaded
            .packages
            .iter()
            .find(|p| p.path == "github.com/a/host")
            .unwrap();
        assert_eq!(npm_pkg.rails, Rails::Npm);
        assert_eq!(stack_pkg.rails, Rails::Stack);
    }
}
