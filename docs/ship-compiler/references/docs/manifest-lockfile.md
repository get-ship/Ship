---
group: Compiler
order: 5
title: Manifest & Lockfile
description: ship.jsonc manifest shape, validation, and the ship.lock dependency lockfile. Package identity, exports, versioned dependencies.
audience: internal
---

# Manifest & Lockfile

Two on-disk files the compiler understands for package management: `.ship/ship.jsonc` (the manifest) and `.ship/ship.lock` (the lockfile). Unrelated to `.ship/ship.state` (workspace state — separate concern, despite the name similarity).

## ship.jsonc — the manifest

`crates/core/compiler/src/manifest.rs`. JSONC primary format. TOML is a legacy path still accepted on read, not written on save.

Three sections:

```jsonc
{
  "module": { "name": "...", "version": "..." },
  "dependencies": { "github.com/foo/bar": "^1.0" },
  "exports": [ "skills/foo", "agents/code" ]
}
```

### ManifestModule

```rust
pub struct ManifestModule {
    pub name: String,          // "github.com/owner/repo" or "@scope/name"
    pub version: String,       // semver, no "v" prefix
    pub description: Option<String>,
    pub license: Option<String>,
    pub authors: Vec<String>,  // "Alice <alice@example.com>"
    // ... more metadata
}
```

Name must match `^[a-z0-9._/@-]+$`. Validation is eager — a malformed manifest fails to parse with an actionable error, not a cryptic deserialize message.

### dependencies

Map of `path → constraint-or-table`. Two shapes:

```jsonc
"dependencies": {
  "github.com/ship-labs/foo": "^1.2",
  "github.com/other/bar": { "version": "2.0", "branch": "main" }
}
```

The short form (string) is a version constraint. The table form carries additional fields for git-specific pinning. See [Registry](/runtime/registry/) for constraint syntax and resolution.

### exports

List of paths the package publishes. Consumers reference these by path after install:

```jsonc
"exports": ["skills/lint", "skills/review", "agents/code"]
```

The compiler publishes a `.ship/ship.lock` entry for each export with its computed hash, so consumers can verify integrity.

### Purity exception

The compiler is pure — no I/O — except for `manifest::from_file` and `lockfile::from_file`, which both read a file. Every other manifest/lockfile function takes already-loaded content.

## ship.lock — the lockfile

`crates/core/compiler/src/lockfile.rs`. **JSONC format** (matches every other `.ship/` config). Records direct resolved dependencies with exact commits and content hashes.

**Not to be confused with `.ship/ship.state`**, which holds per-workspace runtime state (active profile, compiled_at). The lockfile comment calls this out explicitly because the rename from `.ship/ship.lock` (old meaning) to `.ship/ship.state` caused collisions historically.

### Types

```rust
pub struct ShipLock {
    pub version: u32,                // must be 1 for v0.1
    pub packages: Vec<LockPackage>,  // one entry per direct dep
}

pub struct LockPackage {
    pub path: String,                            // "github.com/owner/repo"
    pub version: String,                         // resolved tag or "branch@commit"
    pub commit: String,                          // 40-char hex SHA
    pub hash: String,                            // "sha256:<hex>" combined export hash
    pub export_hashes: BTreeMap<String, String>, // per-export integrity hashes
}
```

`export_hashes` was added in v0.1 for per-export integrity verification. Legacy lockfiles without the field deserialize with an empty map — the `#[serde(default)]` + `skip_serializing_if = "BTreeMap::is_empty"` keeps both directions clean.

### Read / write

- `ShipLock::from_file(path)` — parse from `.ship/ship.lock`. Accepts JSONC (line comments, block comments, trailing commas). Returns error with context on failure.
- `ShipLock::from_jsonc_str(s)` — parse a JSONC string. `from_json_str` is a doc-hidden alias for back-compat with older callers.
- `ShipLock::default()` — fresh lockfile with version=1 and empty packages.
- `to_json_string()` writes deterministic pure JSON — packages sorted by `path` so the output is byte-identical for the same logical content. Comments belong in user-authored configs; the lockfile is tool-authored.

Writers beyond the compiler live in `ship_platform::registry::install_helpers` (`load_lock_if_present`, `write_lock_atomic`, `should_write_lock`). See [Registry](/runtime/registry/).

### Integrity path

Install verifies:

1. For each locked package, fetch the commit.
2. Recompute the package's tree hash via `hash::compute_tree_hash`.
3. Compare against `LockPackage.hash`. Mismatch = error, refuse install.
4. For each export in `export_hashes`, recompute `hash::compute_file_hash` on the exported path. Compare.

Lockfile tampering or upstream retag produces hash mismatch and blocks install.

## Why two files, why these names

- **ship.jsonc** is user-authored. Has comments. JSONC is the default Ship format.
- **ship.lock** is tool-authored. The reader accepts JSONC (line comments, block comments, trailing commas); the writer emits pure JSON for deterministic diffs. Same on-disk extension as every other `.ship/` config, so editor tooling treats it uniformly.
- **ship.state** (not in scope here, but called out to avoid confusion) is workspace runtime state. Used to live at `ship.lock`; renamed to disambiguate.

The lockfile used to be TOML; the reader was migrated to JSONC in commit `94bb54c87` (#187) after lane-spawn parse failures during the eval pipeline dogfood surfaced the format mismatch with the rest of `.ship/`.

## Validation

`manifest.rs` validates at parse time:

- Module name matches `^[a-z0-9._/@-]+$`.
- Version is a valid semver.
- Dependency paths match the name regex.
- Export paths are relative and don't escape the package.

Errors include the field path and the bad value. Not `serde_json::Error` at the user's face.

`lockfile.rs` validates the version == 1 on load. Newer formats would refuse to load rather than silently misinterpret fields.

## Not here

- Dependency **resolution** (constraint → commit) lives in `ship_platform::registry::resolver`, not here. The compiler is pure — it can't do `git ls-remote`.
- **Install** lives in `ship_platform::registry::install`. The compiler produces and validates lockfiles; it doesn't fetch bytes.
- **Hash computation** lives in `ship_platform::registry::hash`. The compiler uses the hashes via lockfile entries but doesn't compute them.

The compiler's job is the format: parse manifests and lockfiles, serialize them back, validate them. The runtime's job is the network + filesystem I/O.

## Related

- [Registry](/runtime/registry/) — dependency resolver, install, hash computation.
- [Compilation Pipeline](/compiler/pipeline/) — manifest feeds into the compile pipeline via `ProjectLibrary`.
