//! Project-shape change watcher (J4).
//!
//! Subscribes to filesystem changes under the project root, debounces them
//! (50ms — the spec's value, smaller than the 200ms `ship-config` watcher
//! default because the cache invalidation half is the cheap, non-event-
//! authoring side; the rest of the policy sits in `cache`), and pushes a
//! [`ResolverInvalidation`] onto a `tokio::sync::watch` channel for the
//! descriptor cache (J5) to consume.
//!
//! ## What this module owns
//!
//! The fs-watch port (`fs-watch::FsWatch`) is **event-free** by design
//! (`fs-watch/src/fs_watch.rs:9-12`: "watching is EVENT-FREE: the port
//! yields raw filesystem deltas"). The port owns the boring mechanism
//! (`notify` wiring, debounce coalescing); the DOMAIN that subscribes
//! owns scope resolution, glob matching, and authorship. The resolver
//! is that domain: this module maps raw `FsDelta`s to
//! [`ResolverInvalidation`]s and nothing else.
//!
//! ## What this module does NOT do
//!
//! - **No kernel events.** The port is event-free; the resolver cache
//!   subscribes directly to the watch channel. A future cell may add
//!   transient `resolver.file_changed` authorship if config reactivity
//!   needs it; today the cache is the only consumer.
//! - **No project walk on each delta.** The watcher does not re-run
//!   `load_project_library` on every FsDelta. The cache is the place
//!   that turns a delta into an eviction (J5); the watcher's job is
//!   to deliver the delta with low latency.
//! - **No path filtering against the spec's surface.** The spec calls
//!   for watching `ship.ts + skills/ + rules/ + .ship/<domains>/`; the
//!   one-watch-on-the-project-root design below covers ALL of those
//!   paths in a single `notify` watcher (the recursive mode descends
//!   into subdirs automatically), with the consumer treating the cap-
//!   relative `rel` as the scope of interest. This matches the port's
//!   "DOMAIN owns scope" contract — the resolver's scope is "the whole
//!   project root", and the cache (the actual owner of the spec's
//!   surface) is the one that decides what to do with each delta.
//!
//! ## Debounce (50ms)
//!
//! The watch primitive (`fs-watch::LocalFsWatch::watch`) accepts a
//! `WatchOpts { recursive, debounce }`. We pass `recursive: true` (the
//! project root contains subdirs) and `debounce: Some(50ms)` (the spec).
//! The port's own debounce coalesces a burst of changes within the
//! window into one `FsDelta` per kind; we DO NOT add a second
//! debounce layer on top — that would be a double-coalesce with
//! surprising latency.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J4J5-watch-cache.md`
//!   (J4 cell).
//! - Census: `census-phase2-resolver.md` Part 3a-3b (the FsWatch port
//!   + ship-config's `spawn_config_watcher` it composes against).
//! - FsWatch contract: `fs-watch/src/fs_watch.rs:1-13` (event-free,
//!   DOMAIN owns scope).
//! - Pattern citation: `ship-config/src/watcher.rs:200-229` — the
//!   `spawn_domain_watch` shape (cap + WatchOpts + spawn) the resolver
//!   mirrors. The 50ms value is the spec's, not the ship-config
//!   default (200ms); the difference is documented at the call site.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use fs_watch::{ChangeKind, FsDelta, FsWatch, LocalFsWatch, WatchOpts};
use ship_substrate::ports::{FsCap, FsMode, FsRoot, SpaceId};
use tokio::sync::watch;
use tokio::task::JoinHandle;

/// The debounce window the resolver's watcher uses. Spec'd at 50ms in
/// `job-spec-p2-J4J5-watch-cache.md` (J4 cell, "50ms debounce"). Smaller
/// than `ship-config::resolve_watcher_debounce()` (200ms default) because
/// the cache invalidation half is cheap — the watcher fires on every
/// touched file, the cache evicts synchronously on the invalidation tick.
pub const RESOLVER_DEBOUNCE: Duration = Duration::from_millis(50);

/// The set of relative paths the resolver cares about under the project
/// root. Listed in the spec as
/// `ship.ts + skills/ + rules/ + .ship/{mcp,permissions,hooks,skills,rules,agents}/`.
///
/// The watcher itself watches the project root recursively (a single
/// `notify` watcher covers every path here). This constant is the
/// SCOPE the cache's invalidation logic uses to decide whether a
/// delta is interesting — the watcher's job is to deliver the delta,
/// the cache's job is to decide what to evict. A path is "in scope"
/// when its first segment matches one of these prefixes (for the
/// `.ship/<domain>/` forms) or equals one of the literal files
/// (`ship.ts` at the root).
pub const WATCHED_ROOTS: &[&str] = &[
    "ship.ts",
    "skills",
    "rules",
    ".ship/mcp.jsonc",
    ".ship/permissions.jsonc",
    ".ship/hooks.jsonc",
    ".ship/skills",
    ".ship/rules",
    ".ship/agents",
];

/// One invalidation tick on the watch channel. Carries the distinct
/// touched paths (project-root-relative) coalesced by the watcher's
/// debounce, plus a monotonic `seq` the cache uses to detect "have I
/// seen this one yet" without reading `paths`.
///
/// The channel starts at `seq = 0` (no invalidation has happened yet).
/// Every coalesced FsDelta increments the seq. A subscriber that
/// wants "any change since I last looked" compares `seq` against the
/// `seq` it last acted on.
#[derive(Debug, Clone)]
pub struct ResolverInvalidation {
    /// The distinct touched paths, project-root-relative (the same
    /// `rel` shape `FsDelta` carries). Coalesced by the watcher's
    /// own debounce; this is the "burst" the cache reacts to.
    pub paths: Vec<String>,
    /// The kind of the last FsDelta in the burst (informational; the
    /// cache treats all three kinds identically — "something changed").
    pub kind: ChangeKind,
    /// Monotonic tick the cache uses to skip duplicate notifications
    /// (the watch channel is `watch::Sender` which re-sends on every
    /// `send`, so a stale subscriber that hasn't `borrow_and_update`
    /// in a while sees the latest value when it next checks).
    pub seq: u64,
}

/// RAII handle owning the watcher's consumer task. Dropping it aborts
/// the task, dropping each `FsDelta` receiver and tearing down the
/// underlying OS watcher.
///
/// The handle holds the `watch::Sender`; the consumer task holds a
/// clone. To listen for invalidations, call [`ResolverWatcher::subscribe`]
/// (clones the `watch::Receiver` cheaply) BEFORE the handle drops.
pub struct ResolverWatcher {
    tasks: Vec<JoinHandle<()>>,
    invalidation: watch::Sender<ResolverInvalidation>,
}

impl ResolverWatcher {
    /// Spawn a watcher over `space_root`. The watch is rooted at
    /// the space root with a recursive `WatchOpts::recursive(50ms)`,
    /// the spec's value. Returns an RAII handle; the consumer task
    /// lives for the handle's lifetime.
    ///
    /// `None` is returned when the watch could not be opened (missing
    /// project root, cap-bootstrap failure). Callers that need a
    /// "watcher-less" mode (e.g. ephemeral test runs) can fall through
    /// to the unwatched path; the cache (J5) is the only consumer
    /// that requires a watcher, and it works without one (the cache
    /// just never sees an invalidation tick).
    pub fn spawn(space_root: &Path) -> Result<Self> {
        let root = space_root.to_path_buf();
        if !root.exists() {
            anyhow::bail!(
                "resolver watcher: project root does not exist: {}",
                root.display()
            );
        }
        let cap = FsCap::new_unchecked(
            root.clone(),
            FsRoot::Space(SpaceId::new("resolver-watcher")),
            FsMode::ReadOnly,
        );
        // Single recursive watch on the project root. notify descends
        // into subdirs (skills/, rules/, .ship/skills/, etc.) and emits
        // cap-relative `rel` for every change. The cache filters to
        // the in-scope paths; the watcher itself doesn't care.
        let opts = WatchOpts::recursive(Some(RESOLVER_DEBOUNCE));
        let watch_impl = LocalFsWatch::new();
        let mut rx = watch_impl
            .watch(&cap, ".", opts)
            .with_context(|| format!("resolver watcher: open watch at {}", root.display()))?;

        // The watch channel starts at "no invalidation" (seq = 0,
        // empty paths). Subscribers see this value on first
        // `borrow_and_update`; a non-empty `paths` is the "something
        // happened" signal.
        let (tx, _initial_rx) = watch::channel(ResolverInvalidation {
            paths: Vec::new(),
            kind: ChangeKind::Modify,
            seq: 0,
        });
        let tx_clone = tx.clone();
        let root_for_task = root.clone();
        let task = tokio::spawn(async move {
            let mut seq: u64 = 0;
            // One FsDelta per kind per debounce window (the port's
            // contract). Coalesce ACROSS kinds by collecting them
            // into a single `ResolverInvalidation` — the cache's
            // eviction logic is "something changed in this space",
            // the kind is informational. We pick the latest kind
            // seen in the burst.
            while let Some(delta) = rx.recv().await {
                seq += 1;
                let in_scope = filter_to_watched(&delta, &root_for_task);
                if in_scope.is_empty() {
                    // Out-of-scope change (e.g. a file under
                    // `<project>/target/`). Skip the tick — no
                    // invalidation. We still bump the seq, but the
                    // consumer's `paths` empty → no-op for the
                    // cache. Actually: we send the tick with empty
                    // paths, because subscribers might want to know
                    // "I saw something, just not in my scope" — but
                    // the cache's eviction is keyed on (space_id,
                    // content_hash) and a same-hash write followed
                    // by an out-of-scope change does NOT require
                    // eviction. So we DON'T send the tick.
                    //
                    // Decision: skip. The watcher's contract is
                    // "the cache invalidates when the descriptor
                    // CHANGES". An out-of-scope change is not a
                    // descriptor change.
                    continue;
                }
                let tick = ResolverInvalidation {
                    paths: in_scope,
                    kind: delta.kind,
                    seq,
                };
                if tx_clone.send(tick).is_err() {
                    // All subscribers dropped — the watcher is being
                    // torn down. The receive loop on `rx` will exit
                    // when the next notify event hits (the port
                    // silently drops the delta because no consumer
                    // is left). We're done.
                    return;
                }
            }
        });

        Ok(Self {
            tasks: vec![task],
            invalidation: tx,
        })
    }

    /// Spawn a watcher that ALSO observes the cache tier's content
    /// (`ShipHome::cache_dir()/objects/`). The cache is the source of
    /// truth for downloaded packages (the #122 dep loader reads
    /// `<cache_root>/objects/<hex>/ship.jsonc`); a `ship install` (or
    /// any other change to the cache) must invalidate the roll-up's
    /// content hash so the J5 cache evicts the stale entry.
    ///
    /// Implementation: a second `LocalFsWatch` task on the cache root
    /// feeds the same `watch::Sender<ResolverInvalidation>` channel the
    /// space-root watch feeds. The cache subscriber (J5) sees ticks
    /// from either source and evicts on any non-empty `paths`.
    ///
    /// The cache-root watch fires on the recursive `notify` over the
    /// `objects/` tree; the `WATCHED_ROOTS` scope check still applies
    /// to space-root deltas (cache deltas bypass the scope check —
    /// ANY change under `objects/` is in scope for the roll-up).
    pub fn spawn_with_cache(space_root: &Path) -> Result<Self> {
        let mut watcher = Self::spawn(space_root)?;
        let cache_root = ship_substrate::ShipHome::from_env()
            .cache_dir()
            .join("objects");
        if cache_root.exists() {
            watcher.spawn_extra_watch(&cache_root)?;
        }
        Ok(watcher)
    }

    /// Spawn an additional watch on `extra_root`. The extra watch's
    /// deltas are fed into the same invalidation channel as the
    /// primary watch. The extra watch's deltas bypass the
    /// `WATCHED_ROOTS` scope check — ANY change under `extra_root`
    /// is in scope (used for the cache tier where every file is
    /// content-addressed and every change is a potential roll-up
    /// invalidation).
    fn spawn_extra_watch(&mut self, extra_root: &Path) -> Result<()> {
        let root = extra_root.to_path_buf();
        let cap = FsCap::new_unchecked(
            root.clone(),
            FsRoot::Space(SpaceId::new("resolver-cache-watcher")),
            FsMode::ReadOnly,
        );
        let opts = WatchOpts::recursive(Some(RESOLVER_DEBOUNCE));
        let watch_impl = LocalFsWatch::new();
        let mut rx = watch_impl
            .watch(&cap, ".", opts)
            .with_context(|| format!("resolver watcher: open cache watch at {}", root.display()))?;

        let tx = self.invalidation.clone();
        let task = tokio::spawn(async move {
            // Cache-tier deltas bypass the `WATCHED_ROOTS` scope check:
            // every change under the cache is in scope. The watcher's
            // contract is "something in the cache changed → roll-up
            // invalidation".
            let mut seq: u64 = 0;
            while let Some(delta) = rx.recv().await {
                seq += 1;
                // `FsDelta::rel` is a `Vec<String>` of distinct touched
                // paths (cap-relative). The cache subscriber treats
                // any non-empty tick as an invalidation.
                if delta.rel.is_empty() {
                    continue;
                }
                let tick = ResolverInvalidation {
                    paths: delta.rel,
                    kind: delta.kind,
                    seq,
                };
                if tx.send(tick).is_err() {
                    return;
                }
            }
        });
        // The handle holds every spawned watch task; Drop aborts them
        // all so the underlying `notify` watchers release cleanly.
        self.tasks.push(task);
        Ok(())
    }

    /// Borrow a fresh receiver on the invalidation channel. Cheap
    /// (clones a `watch::Receiver`); callers should hold the
    /// receiver for the duration they want to observe invalidations.
    pub fn subscribe(&self) -> watch::Receiver<ResolverInvalidation> {
        self.invalidation.subscribe()
    }
}

impl Drop for ResolverWatcher {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
///
/// The cap is rooted at the project root, so `rel` is the
/// project-relative path verbatim. An in-scope path is one whose
/// first segment matches one of [`WATCHED_ROOTS`] (for the
/// `.ship/<domain>/` forms) or whose full path is `ship.ts` (the
/// manifest file at the project root).
fn filter_to_watched(delta: &FsDelta, _root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for rel in &delta.rel {
        if is_in_scope(rel) {
            out.push(rel.clone());
        }
    }
    out
}

/// `true` when `rel` is one of the watched paths or under one of the
/// watched subdirs. The check is conservative: `rel` must START with
/// one of the spec's prefixes.
///
/// `ship.ts` is matched as an exact path; `.ship/<domain>` matches
/// anything under that prefix. `skills/` and `rules/` at the project
/// root are matched as prefixes (covers both the default-shape
/// `skills/<id>/...` AND the override `skills/` at the project root).
fn is_in_scope(rel: &str) -> bool {
    // Normalize: notify sometimes returns "./ship.ts" or absolute
    // paths. The cap confinement strips the latter; we just strip
    // the leading "./" for matching.
    let trimmed = rel.strip_prefix("./").unwrap_or(rel);
    if trimmed.is_empty() {
        return false;
    }
    // Exact match: the manifest file at the project root.
    if trimmed == "ship.ts" {
        return true;
    }
    // Prefix match: any watched subdir.
    const PREFIXES: &[&str] = &[
        "skills/",
        "rules/",
        ".ship/mcp.jsonc",
        ".ship/permissions.jsonc",
        ".ship/hooks.jsonc",
        ".ship/skills/",
        ".ship/rules/",
        ".ship/agents/",
    ];
    PREFIXES.iter().any(|p| trimmed.starts_with(p))
        // The `.ship/mcp.jsonc` etc. are FILES, not directories —
        // match exactly when the path equals the literal, but also
        // accept any change under `.ship/` and let the cache filter
        // further. For Phase 2, a subdir-prefix match is enough —
        // the spec calls out the watched set, not the exact files.
        || trimmed.starts_with(".ship/")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn in_scope_recognizes_watched_paths() {
        // Spec's exact list.
        assert!(is_in_scope("ship.ts"));
        assert!(is_in_scope("skills/foo/SKILL.md"));
        assert!(is_in_scope("rules/style.md"));
        assert!(is_in_scope(".ship/mcp.jsonc"));
        assert!(is_in_scope(".ship/permissions.jsonc"));
        assert!(is_in_scope(".ship/hooks.jsonc"));
        assert!(is_in_scope(".ship/skills/foo/SKILL.md"));
        assert!(is_in_scope(".ship/rules/style.md"));
        assert!(is_in_scope(".ship/agents/demo.md"));
    }

    #[test]
    fn in_scope_rejects_unrelated_paths() {
        // Project build artefacts and runtime files are NOT in scope.
        assert!(!is_in_scope("target/debug/foo"));
        assert!(!is_in_scope("node_modules/x/package.json"));
        assert!(!is_in_scope("README.md"));
        assert!(!is_in_scope(""));
    }

    #[test]
    fn in_scope_handles_dot_slash_prefix() {
        // notify sometimes yields "./foo" — normalize.
        assert!(is_in_scope("./ship.ts"));
        assert!(is_in_scope("./skills/x"));
    }

    #[test]
    fn debounce_constant_matches_spec() {
        // The spec is the citation. A change here means the spec
        // changed; update the docstring in lockstep.
        assert_eq!(RESOLVER_DEBOUNCE, Duration::from_millis(50));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn watcher_spawns_and_subscribe_yields_initial_value() {
        let tmp = tempfile::tempdir().unwrap();
        let w = ResolverWatcher::spawn(tmp.path()).expect("watcher spawns");
        let mut rx = w.subscribe();
        // The initial value is seq=0, empty paths.
        let initial = rx.borrow_and_update().clone();
        assert_eq!(initial.seq, 0);
        assert!(initial.paths.is_empty());
        // Drop the watcher → the consumer task is aborted.
        drop(w);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn watcher_fires_on_in_scope_change() {
        let tmp = tempfile::tempdir().unwrap();
        // Lay out a minimal project so the in-scope change is a
        // real write the watch primitive can see.
        std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        let w = ResolverWatcher::spawn(tmp.path()).expect("watcher spawns");
        let mut rx = w.subscribe();

        // Touch an in-scope file. The 50ms debounce window is
        // small; give notify + the spawn a generous headroom so
        // the assertion isn't flaky on slow CI.
        tokio::time::sleep(Duration::from_millis(80)).await;
        std::fs::write(
            tmp.path().join(".ship").join("mcp.jsonc"),
            "{\"mcp\":{\"servers\":{}}}",
        )
        .unwrap();
        // Wait past the debounce + a margin for the consumer to
        // forward.
        let mut got_invalidation = false;
        let deadline = std::time::Instant::now() + Duration::from_millis(800);
        while std::time::Instant::now() < deadline {
            if rx.changed().await.is_err() {
                break;
            }
            let tick = rx.borrow_and_update().clone();
            if !tick.paths.is_empty() {
                got_invalidation = true;
                assert!(tick.seq >= 1, "seq should advance past 0");
                // The rel is project-root-relative, the cap confines
                // to the project root so this is the literal path.
                assert!(
                    tick.paths
                        .iter()
                        .any(|p| p.contains("mcp.jsonc") || p.contains(".ship/")),
                    "in-scope path was not in tick: {:?}",
                    tick.paths
                );
                break;
            }
        }
        assert!(
            got_invalidation,
            "watcher did not fire on an in-scope change within 800ms"
        );
        drop(w);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn watcher_silently_drops_out_of_scope_change() {
        let tmp = tempfile::tempdir().unwrap();
        let w = ResolverWatcher::spawn(tmp.path()).expect("watcher spawns");
        let mut rx = w.subscribe();

        // Write to an out-of-scope path.
        tokio::time::sleep(Duration::from_millis(80)).await;
        std::fs::write(tmp.path().join("README.md"), "noise").unwrap();

        // Wait long enough for the debounce window + a margin.
        // Expectation: no tick arrives (the consumer drops the
        // delta because `filter_to_watched` returns empty).
        // We use a `tokio::time::timeout` to bound the wait —
        // success = the timeout fired (no tick).
        let outcome = tokio::time::timeout(Duration::from_millis(400), async {
            loop {
                if rx.changed().await.is_err() {
                    return Err::<(), &'static str>("watcher dropped");
                }
                let tick = rx.borrow_and_update().clone();
                if !tick.paths.is_empty() {
                    return Err("out-of-scope change produced a tick");
                }
            }
        })
        .await;
        assert!(
            outcome.is_err(),
            "watcher fired on out-of-scope change (expected no tick)"
        );
        drop(w);
    }

    /// `spawn_with_cache` adds a second watch on the XDG cache's
    /// `objects/` dir. The spec requires the watcher extend over the
    /// cache tier's content hashes — a `ship install` (which adds a
    /// new `<hex>/` to the cache) must trigger an invalidation.
    ///
    /// Test: the cache root may not exist (XDG home unset, no
    /// `~/.ship/cache/objects/` yet), and `spawn_with_cache` must
    /// still succeed (degraded to the space-root-only watch).
    #[tokio::test(flavor = "current_thread")]
    async fn spawn_with_cache_succeeds_when_cache_root_missing() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        // The cache root won't exist under `ShipHome::for_test` with
        // an isolated home; `spawn_with_cache` should still succeed.
        let w = ResolverWatcher::spawn_with_cache(tmp.path())
            .expect("spawn_with_cache must succeed even without cache");
        // Subscribe to verify the channel is still functional.
        let mut rx = w.subscribe();
        let initial = rx.borrow_and_update().clone();
        assert_eq!(initial.seq, 0);
        assert!(initial.paths.is_empty());
        drop(w);
    }
}
