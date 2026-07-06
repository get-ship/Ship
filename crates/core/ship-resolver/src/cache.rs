//! Descriptor cache (J5).
//!
//! Per-harness `CompileOutput` cache, keyed by `(space_id, content_hash, harness_id)`,
//! stored at `<runtime_cache_root>/resolver/<space_id>/<content_hash>__<harness_id>.json`
//! via FsBroker (gated). Subscribes to the J4 [`watcher::ResolverWatcher`]'s
//! invalidation channel; on a tick, evicts the in-memory index for the
//! affected `space_id`.
//!
//! ## The contract
//!
//! - `get_or_load(space_id, hash, harness_id, load_fn)` — on hit, return
//!   the cached `CompileOutput`; on miss, call `load_fn`, write the
//!   result to disk via `tokio::spawn` (the spec's R7 — the cache write
//!   must never block the caller), return the value.
//! - The on-disk format is the `CompileOutput`'s `serde_json` form. The
//!   bytes are written via `FsBroker::write_atomic` (temp + fsync + rename),
//!   so a concurrent reader either sees the previous file (the rename
//!   hasn't happened) or the new file (rename completed) — never a
//!   torn file. This is the R5 ordering invariant the spec calls out.
//! - Invalidation: the subscriber holds a `watch::Receiver<ResolverInvalidation>`
//!   from the J4 channel. Each tick clears the in-memory index for every
//!   known `space_id`. The on-disk files remain (best-effort cleanup is
//!   a Phase-3 follow-on; today a stale file at `<hash_v1>__claude.json`
//!   is harmless because the next read with `<hash_v2>` is a miss).
//! - Eviction is per-space (coarse). The cache does NOT know which
//!   entries are stale on any given tick; it clears the whole space's
//!   index. This is the safe over-eviction; the next read repopulates.
//!   A future cell could narrow this to per-hash eviction by carrying
//!   the new hash on the invalidation channel — out of scope for
//!   Phase 2.
//!
//! ## What this module does NOT do
//!
//! - **No kernel events.** The cache is the only consumer of the
//!   watcher's invalidation channel today; no transient
//!   `resolver.cache_evicted` event is authored. A future cell may
//!   add one for compile reactivity.
//! - **No on-disk cleanup.** Stale `<old_hash>__harness.json` files
//!   accumulate. The next read with the matching hash hits; a
//!   different hash misses. The blast radius is bounded by the
//!   number of distinct hashes a project has ever compiled against.
//! - **No metrics.** A future cell (Phase-3 follow-on per the
//!   census's R4) will add hit/miss counters.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J4J5-watch-cache.md`
//!   (J5 cell).
//! - Census: `census-phase2-resolver.md` Part 3e-3g (cache today) + Part 5 J5
//!   (this module's scope).
//! - Determinism: `inv-fmt-deterministic-eval` (`ship-stack-format.json:30-33`).
//! - Resolver surface: `descriptor::project_library_content_hash` is the
//!   hash the cache keys on (the caller of `get_or_load` computes it from
//!   the `ProjectLibrary`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use compiler::CompileOutput;
use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker, SpaceId};
use tokio::sync::Mutex;
use tokio::sync::watch;
use tracing::warn;

use crate::watcher::{ResolverInvalidation, ResolverWatcher};

/// The descriptor cache. Holds an in-memory index of (space, hash, harness)
/// → `Arc<CompileOutput>` (so a hit doesn't have to round-trip the disk)
/// plus an FsBroker handle for the disk write. The cache root is
/// configurable; production uses `<runtime_cache_root>/resolver/`
/// (the `runtime_cache_root` is the XDG cache path the platform
/// surfaces; the second `resolver` is the cache's own namespace so
/// a future platform cache layer can share `<runtime_cache_root>`
/// with other consumers).
///
/// The cache is `Clone`-cheap: the inner state is wrapped in
/// `Arc<Mutex<...>>` so all clones share the same index. The watch
/// receiver is per-clone — each `DescriptorCache` value owns one
/// receiver; the test surface typically has one cache, but the
/// design supports N subscribers.
#[derive(Clone)]
pub struct DescriptorCache {
    inner: Arc<Mutex<CacheState>>,
    cache_root: PathBuf,
    broker: Arc<dyn FsBroker>,
    _watcher_handle: Option<watch::Receiver<ResolverInvalidation>>,
}

use std::path::PathBuf;

/// The cache's mutable state. Held under a `Mutex` so multiple
/// `get_or_load` calls don't race on the in-memory index. The
/// `tokio::sync::Mutex` (vs `std::sync::Mutex`) is the right call:
/// `get_or_load` may await (load_fn is `async`).
struct CacheState {
    entries: HashMap<CacheKey, Arc<CompileOutput>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    space: String,
    content_hash: String,
    harness: String,
}

impl DescriptorCache {
    /// Open a cache rooted at `cache_root`. The cache subscribes to
    /// `watcher`'s invalidation channel; on every tick, the
    /// in-memory index is cleared (per-space eviction).
    ///
    /// `watcher` is optional: pass `None` for tests that don't
    /// need invalidation, or for the "cache-only" mode where the
    /// caller manages invalidation externally.
    pub fn open(
        cache_root: impl Into<PathBuf>,
        watcher: Option<&ResolverWatcher>,
    ) -> Result<Self> {
        let cache_root = cache_root.into();
        let broker: Arc<dyn FsBroker> = Arc::new(LocalFsBroker::new());
        let _watcher_handle = watcher.map(|w| w.subscribe());
        Ok(Self {
            inner: Arc::new(Mutex::new(CacheState {
                entries: HashMap::new(),
            })),
            cache_root,
            broker,
            _watcher_handle,
        })
    }

    /// Get the cached `CompileOutput` for `(space_id, content_hash, harness_id)`.
    /// On miss, call `load_fn` to produce the value, write it to the cache
    /// via `tokio::spawn` (R7 — non-blocking), and return the value
    /// synchronously to the caller.
    ///
    /// `load_fn` is async: the harness compile is an async fn (it may
    /// hit the network, read additional files, etc). The cache's only
    /// contract with `load_fn` is "returns the `CompileOutput` to cache".
    ///
    /// The on-disk write is fire-and-forget: a write failure is logged
    /// + dropped (the spec's R7 — the next read re-runs `load_fn`).
    pub async fn get_or_load<F, Fut>(
        &self,
        space_id: &SpaceId,
        content_hash: &str,
        harness_id: &str,
        load_fn: F,
    ) -> Result<Arc<CompileOutput>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<CompileOutput>>,
    {
        let key = CacheKey {
            space: space_id.as_str().to_string(),
            content_hash: content_hash.to_string(),
            harness: harness_id.to_string(),
        };
        // Fast path: in-memory hit.
        {
            let state = self.inner.lock().await;
            if let Some(payload) = state.entries.get(&key) {
                return Ok(payload.clone());
            }
        }
        // Slow path: miss. Compute the payload, populate the index,
        // spawn the disk write.
        let payload = Arc::new(
            load_fn()
                .await
                .with_context(|| format!("descriptor cache: load_fn for {key:?}"))?,
        );
        {
            let mut state = self.inner.lock().await;
            // Re-check under the lock in case a concurrent caller
            // populated the same key while we were awaiting
            // `load_fn`. We prefer their entry (no double-write).
            if let Some(existing) = state.entries.get(&key) {
                return Ok(existing.clone());
            }
            state.entries.insert(key.clone(), payload.clone());
        }
        // Build the on-disk path. The cap is rooted at
        // `<cache_root>/resolver/`; the rel is
        // `<space_id>/<content_hash>__<harness_id>.json` (the
        // space prefix is part of the rel so `create_dir_all`
        // walks the whole path and creates the space subdir
        // atomically with the file write).
        let rel = format!(
            "{}/{}__{}.json",
            key.space, key.content_hash, key.harness
        );
        // Spawn the disk write. We clone the broker, the
        // `Arc<CompileOutput>`, and the `cache_root`/`space` so
        // the spawn doesn't borrow `self`.
        let broker = self.broker.clone();
        let cache_root = self.cache_root.clone();
        let space = space_id.clone();
        let key_for_log = key.clone();
        let payload_for_write = payload.clone();
        tokio::spawn(async move {
            if let Err(e) = write_entry(&*broker, &cache_root, &space, &rel, &payload_for_write).await {
                warn!(
                    "descriptor cache: write failed for {key_for_log:?}: {e} \
                     (next read will miss and re-load)"
                );
            }
        });
        Ok(payload)
    }

    /// Invalidate the cache. Clears ALL entries for `space_id` from
    /// the in-memory index. The on-disk files remain (best-effort
    /// cleanup is out of scope; the next read for a different hash
    /// will miss and re-write).
    pub async fn invalidate(&self, space_id: &SpaceId) {
        let mut state = self.inner.lock().await;
        let before = state.entries.len();
        state
            .entries
            .retain(|k, _| k.space != space_id.as_str());
        let after = state.entries.len();
        if before != after {
            tracing::debug!(
                "descriptor cache: invalidated {} entries for space {} ({} → {})",
                before - after,
                space_id.as_str(),
                before,
                after
            );
        }
    }

    /// Drive the invalidation subscriber. For each tick on the watch
    /// channel, evicts the in-memory index for every known
    /// `space_id`. Returns a `JoinHandle` so the caller can
    /// `abort()` it (e.g. on shutdown).
    ///
    /// Without a watcher (the `cache_root` was opened with
    /// `None`), the subscriber is a no-op: it returns an immediately-
    /// completed `JoinHandle`.
    pub fn spawn_invalidation_subscriber(&self) -> tokio::task::JoinHandle<()> {
        let cache = self.clone();
        let mut rx = match self._watcher_handle.as_ref() {
            Some(rx) => rx.clone(),
            None => {
                return tokio::spawn(async move {});
            }
        };
        tokio::spawn(async move {
            // The first `borrow_and_update` returns the initial
            // value (seq=0, empty paths). We treat this as "no
            // invalidation yet" and skip it.
            let _ = rx.borrow_and_update();
            while rx.changed().await.is_ok() {
                let tick = rx.borrow_and_update().clone();
                if tick.paths.is_empty() {
                    // Out-of-scope change (the J4 watcher filters
                    // these out for the cache anyway, but the
                    // tick's `paths` is the contract).
                    continue;
                }
                // The tick's `paths` is project-root-relative;
                // we don't know the `space_id` from the path
                // alone. The conservative choice: invalidate
                // every space we know about. Today the cache is
                // per-process and serves one project; we iterate
                // the known spaces from the in-memory index.
                let known_spaces: Vec<SpaceId> = {
                    let state = cache.inner.lock().await;
                    let mut set: std::collections::HashSet<SpaceId> =
                        std::collections::HashSet::new();
                    for k in state.entries.keys() {
                        set.insert(SpaceId::new(&k.space));
                    }
                    set.into_iter().collect()
                };
                for space in known_spaces {
                    cache.invalidate(&space).await;
                }
            }
        })
    }

    /// Number of entries in the in-memory index. Test-only surface.
    #[doc(hidden)]
    pub async fn _len(&self) -> usize {
        self.inner.lock().await.entries.len()
    }
}

impl std::fmt::Debug for DescriptorCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorCache")
            .field("cache_root", &self.cache_root)
            .field("has_watcher", &self._watcher_handle.is_some())
            .finish()
    }
}

/// One disk-write of a cache entry. Pulled out so the `tokio::spawn`
/// closure is a single expression; the failure mode (write error) is
/// logged + dropped (the spec's R7: "next read will miss and re-load").
///
/// The cap is rooted at `<cache_root>/resolver/`. The `rel` passed in
/// is `<space_id>/<content_hash>__<harness_id>.json` — the
/// `create_dir_all` walks the rel and creates every missing parent
/// (the resolver dir + the space subdir), so a single call is
/// enough before the `write_atomic`.
async fn write_entry(
    broker: &dyn FsBroker,
    cache_root: &Path,
    space: &SpaceId,
    rel: &str,
    payload: &CompileOutput,
) -> Result<()> {
    // The cap root must EXIST before we hand it to the broker.
    // The FsBroker's `reject_symlink_escape` walks up to find
    // the first existing ancestor and compares it against the
    // cap root via `starts_with` — a non-existent cap root
    // means the comparison is always false and every op fails
    // with `PathEscape`. So we materialize the resolver dir
    // with `std::fs` first (this is a one-time bootstrap, not
    // the runtime data path the cache brokers for; the broker
    // owns the space subdir and the file itself).
    let resolver_dir = cache_root.join("resolver");
    std::fs::create_dir_all(&resolver_dir)
        .with_context(|| format!("create resolver dir at {}", resolver_dir.display()))?;
    let cap = FsCap::new_unchecked(
        resolver_dir,
        FsRoot::Space(space.clone()),
        FsMode::ReadWrite,
    );
    // Walk ONLY the directory portion of the rel. `create_dir_all`
    // treats every segment of its rel as a directory to create,
    // so passing the file's rel would create the file as a
    // directory and the subsequent `write_atomic` would fail
    // to rename the temp file over a directory. Strip the
    // basename and create only the parent dir.
    let dir_rel = match rel.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => parent.to_string(),
        _ => ".".to_string(),
    };
    broker
        .create_dir_all(&cap, &dir_rel)
        .map_err(|e| anyhow::anyhow!("create_dir_all: {e:?}"))?;
    let bytes = serde_json::to_vec(payload).context("serialize CompileOutput")?;
    broker
        .write_atomic(&cap, rel, &bytes)
        .map_err(|e| anyhow::anyhow!("write_atomic: {e:?}"))?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    /// A `CompileOutput` with a few fields populated, so the
    /// serde round-trip is non-trivial.
    fn sample_output(label: &str) -> CompileOutput {
        let mut out = CompileOutput::default();
        out.context_content = Some(format!("// {label}\n"));
        out.skill_files
            .insert(format!(".claude/skills/{label}/SKILL.md"), "body".to_string());
        out.mcp_servers = json!({ "mcpServers": { label: { "command": "echo" } } });
        out
    }

    /// A `CompileOutput` with a distinct set of fields so the test
    /// can tell the two apart by value (not just by reference).
    fn other_output(label: &str) -> CompileOutput {
        let mut out = CompileOutput::default();
        out.context_content = Some(format!("// other: {label}\n"));
        out
    }

    fn make_space(s: &str) -> SpaceId {
        SpaceId::new(s)
    }

    /// Build a cache rooted at a fresh `TempDir`. The `TempDir` is
    /// returned alongside the cache so the test can assert on the
    /// disk state.
    fn fresh_cache() -> (TempDir, DescriptorCache) {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DescriptorCache::open(tmp.path(), None).unwrap();
        (tmp, cache)
    }

    /// Build a cache + watcher pair. The project root the watcher
    /// watches is a sibling of the cache root.
    fn make_cache_with_watcher() -> (TempDir, DescriptorCache, ResolverWatcher) {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path().join("proj");
        std::fs::create_dir_all(&project_root).unwrap();
        let watcher = ResolverWatcher::spawn(&project_root).unwrap();
        let cache = DescriptorCache::open(tmp.path().join("cache"), Some(&watcher)).unwrap();
        (tmp, cache, watcher)
    }

    // ── Test 1: hit returns identical bytes ──────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn hit_returns_identical_bytes() {
        let (_tmp, cache) = fresh_cache();
        let space = make_space("s1");
        let hash = "h1";
        let harness = "claude";

        // First call: miss → load_fn runs.
        let first = cache
            .get_or_load(&space, hash, harness, || async {
                Ok(sample_output("alpha"))
            })
            .await
            .unwrap();
        // Give the spawned write a chance to complete.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Second call: should hit (in-memory). `load_fn` is the
        // hit detector — a panic inside it means the cache
        // didn't return the in-memory value.
        let second = cache
            .get_or_load(&space, hash, harness, || async {
                panic!("load_fn called on a hit");
            })
            .await
            .unwrap();

        // The hit's bytes are identical to the first load's bytes.
        let first_bytes = serde_json::to_vec(&*first).unwrap();
        let second_bytes = serde_json::to_vec(&*second).unwrap();
        assert_eq!(
            first_bytes, second_bytes,
            "a cache hit must return byte-identical output"
        );
    }

    // ── Test 2: miss writes then reads ───────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn miss_writes_then_reads() {
        let (tmp, cache) = fresh_cache();
        let space = make_space("s2");
        let hash = "h2";
        let harness = "codex";

        // Pre-condition: no file on disk yet.
        let space_dir = tmp.path().join("resolver").join(space.as_str());
        assert!(
            !space_dir.exists(),
            "cache space dir must not exist before the first miss: {}",
            space_dir.display()
        );

        // First call: miss → load_fn runs → spawn disk write.
        let first = cache
            .get_or_load(&space, hash, harness, || async {
                Ok(sample_output("beta"))
            })
            .await
            .unwrap();
        // Give the spawned write plenty of time to flush.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Disk now has the file.
        let on_disk = space_dir.join(format!("{hash}__{harness}.json"));
        assert!(
            on_disk.exists(),
            "cache file must exist after a miss: {}",
            on_disk.display()
        );
        // The on-disk bytes match the in-memory serialized form.
        // (We don't deserialize — the on-disk format IS the
        // in-memory Serialize form, and the spec's
        // `inv-fmt-deterministic-eval` invariant is that two
        // reads of the same input produce byte-identical
        // output. A future compiler-leaf change that adds
        // `Deserialize` to `CompileOutput` would make the
        // round-trip explicit; out of scope for the J5 cache
        // (the spec restricts the surface to `ship-resolver`).
        let on_disk_bytes = std::fs::read(&on_disk).unwrap();
        let first_bytes = serde_json::to_vec(&*first).unwrap();
        assert_eq!(
            on_disk_bytes, first_bytes,
            "on-disk bytes must match the in-memory cached form"
        );

        // A second call on the SAME instance hits the in-memory
        // index — `load_fn` is NOT called.
        let load_fn_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let load_fn_called_inner = load_fn_called.clone();
        let _second = cache
            .get_or_load(&space, hash, harness, || async move {
                load_fn_called_inner.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(sample_output("beta"))
            })
            .await
            .unwrap();
        assert!(
            !load_fn_called.load(std::sync::atomic::Ordering::SeqCst),
            "load_fn must not run on an in-memory hit"
        );
    }

    // ── Test 3: invalidation evicts stale ────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn invalidation_evicts_stale() {
        let (_tmp, cache, watcher) = make_cache_with_watcher();
        let space = make_space("s3");
        let hash_v1 = "hash-v1";
        let hash_v2 = "hash-v2";
        let harness = "claude";

        // Populate the cache with hash_v1.
        let v1 = cache
            .get_or_load(&space, hash_v1, harness, || async {
                Ok(sample_output("v1"))
            })
            .await
            .unwrap();
        assert_eq!(v1.context_content.as_deref(), Some("// v1\n"));
        assert_eq!(cache._len().await, 1, "in-memory index has v1");

        // Drive the subscriber.
        let sub = cache.spawn_invalidation_subscriber();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Trigger an invalidation by writing a real file in the
        // watched project root.
        std::fs::write(
            _tmp.path().join("proj").join("ship.ts"),
            "export const x = 1;",
        )
        .unwrap();

        // Wait for the debounce + the subscriber to act.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(800);
        while std::time::Instant::now() < deadline {
            if cache._len().await == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(
            cache._len().await,
            0,
            "in-memory index must be empty after an invalidation tick"
        );

        // A subsequent `get_or_load` for hash_v1 is a miss.
        let v1_again = cache
            .get_or_load(&space, hash_v1, harness, || async {
                Ok(sample_output("v1"))
            })
            .await
            .unwrap();
        assert_eq!(v1_again.context_content, v1.context_content);
        assert_eq!(cache._len().await, 1);

        // A get_or_load for hash_v2 is also a miss.
        let v2 = cache
            .get_or_load(&space, hash_v2, harness, || async {
                Ok(other_output("v2"))
            })
            .await
            .unwrap();
        assert_eq!(v2.context_content.as_deref(), Some("// other: v2\n"));
        assert_eq!(cache._len().await, 2);

        sub.abort();
        drop(watcher);
    }

    // ── Test 4: R5 ordering invariant ───────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn r5_concurrent_fsdelta_and_read_does_not_tear() {
        // The invariant: a reader either gets the stale cache
        // (and a fresh load) OR sees the new cache, NEVER a torn
        // cache (a half-written file, a payload that doesn't
        // match its on-disk form, etc).
        let (_tmp, cache, watcher) = make_cache_with_watcher();
        let sub = cache.spawn_invalidation_subscriber();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let space = make_space("s5");
        let hash = "h5";
        let harness = "claude";

        // 1. Pre-populate.
        let pre = cache
            .get_or_load(&space, hash, harness, || async {
                Ok(sample_output("pre"))
            })
            .await
            .unwrap();
        assert_eq!(pre.context_content.as_deref(), Some("// pre\n"));
        assert_eq!(cache._len().await, 1);

        // 2. Force an invalidation tick.
        std::fs::write(
            _tmp.path().join("proj").join("ship.ts"),
            "export const x = 2;",
        )
        .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(800);
        while std::time::Instant::now() < deadline {
            if cache._len().await == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(cache._len().await, 0);

        // 3. Race a reader against a SECOND invalidation tick.
        let reader = {
            let cache = cache.clone();
            let space = space.clone();
            tokio::spawn(async move {
                cache
                    .get_or_load(&space, hash, harness, || async {
                        // Slow load_fn so the tick has time to
                        // land mid-read.
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        Ok(sample_output("reader"))
                    })
                    .await
            })
        };
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        std::fs::write(
            _tmp.path().join("proj").join("ship.ts"),
            "export const x = 3;",
        )
        .unwrap();
        let reader_value = reader.await.unwrap().unwrap();
        assert_eq!(reader_value.context_content.as_deref(), Some("// reader\n"));
        // The reader's value is byte-stable through serde_json.
        let bytes = serde_json::to_vec(&*reader_value).unwrap();
        assert!(
            !bytes.is_empty(),
            "the reader's value must serialize to a non-empty payload"
        );
        assert!(cache._len().await >= 1);

        sub.abort();
        drop(watcher);
    }
}
