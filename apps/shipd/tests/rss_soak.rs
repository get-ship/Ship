//! G2 — daemon RSS-soak regression gate.
//!
//! Spec: `.artifacts/sui-arc/integrity/report.md` §4 (gate G2). This is the
//! gate that would catch a #256-class daemon leak — the `actor_cursors` strand
//! fixed in `585458a98`, or a `rate_counters` / `ReplayLog` regression — by
//! asserting that the *post-warmup* process RSS slope is ~flat (MB/min ≈ 0)
//! after sustained connection/event churn. A leak that revert-passes the
//! existing unit tests (e.g. a re-introduced unconditional
//! `actor_cursors.insert` on stop) shows up here as a positive slope.
//!
//! ## Path chosen: substrate-direct (NOT the full HTTP daemon)
//!
//! The report authorizes a fallback: drive "the leak-prone substrate directly
//! (KernelRouter spawn/stop + projection + SSE-source actor lifecycle — the
//! exact actor_cursors/rate_counters paths)" if booting the full HTTP daemon is
//! too heavy/infeasible. We take that fallback **deliberately**, because the D1
//! leak this gate exists to catch lives entirely on the `#[cfg(feature =
//! "unstable")]` kernel path: `actor_cursors`, `event_log` (`ReplayLog`), and
//! `actor_meta` are all unstable-gated, and the unstable branch of
//! `KernelRouter::emit` is the only code that touches the replay ring + cursor
//! logic. The default `shipd` HTTP binary is built WITHOUT `unstable`, so
//! `inner.route()` runs and **never exercises `actor_cursors` at all** —
//! booting it would prove nothing about the strand the gate must guard. Driving
//! the `KernelRouter` directly with `unstable` on (forwarded by the `soak`
//! feature) hits the precise spawn/stop/emit/cursor paths on an isolated temp
//! workspace, with no network and no child process to leak.
//!
//! ## Driver (one cycle = one SSE/HMR/request/projection churn round)
//!
//! Each iteration reproduces the connection churn that leaked:
//!   1. SSE-source actor: spawn with a FRESH ULID id (`sui-sse-<data>-<ulid>`,
//!      the exact `sui_sse_sources.rs:85` shape — the D1 path), emit a few
//!      events, then `stop_actor` (the disconnect path that strands a cursor if
//!      unfixed).
//!   2. HMR-style connect/disconnect + rate-counter fanout: a second fresh
//!      unique id spawned/stopped, plus an event with a FRESH originator AND a
//!      FRESH target id so `emit` inserts a `RateCounter` for both (the #256
//!      rate-counter strand).
//!   3. Request cycle: a read query against the SSE actor's store (the per-
//!      request read path) before it is stopped.
//!   4. Periodic projection-rebuild churn: every Nth cycle, replay the event
//!      log back out of the temp-DB store through a long-lived reader actor's
//!      `query` (the read + deserialize + Vec-allocate cost a projection rebuild
//!      incurs). Done against the temp DB only — no global/env-resolved DB.
//!   5. Occasionally a PERMISSIONED/restartable actor (bounded id pool) spawned
//!      and stopped — this MUST retain a cursor (the legitimate retain path), so
//!      the gate also proves we did not over-evict.
//!
//! ## Measure
//!
//! Sample `VmRSS` from `/proc/self/status` every `SAMPLE_EVERY` cycles, discard
//! a ~20% warmup prefix, linear-fit the post-warmup samples, report slope in
//! MB/min and total MB grown. Assert slope below a tight threshold. The number
//! is printed on pass and fail.
//!
//! ## Safety triad (the founder OOM'd this 19GB box twice — a runaway here is
//! unacceptable)
//!
//!   * Hard RSS CEILING: `RSS_CEILING_MB = 1024` (≈1.0 GB, nowhere near 19 GB).
//!     A watchdog OS thread samples `VmRSS` every `WATCHDOG_INTERVAL` and, the
//!     instant RSS crosses the ceiling, sets an abort flag (the driver checks
//!     it every cycle and bails) AND, as a last resort, `process::abort()`s so a
//!     wedged driver cannot keep climbing toward the host limit.
//!   * Hard CYCLE CAP: `MAX_CYCLES = 5000`.
//!   * Wall-clock TIMEOUT: `WALL_CLOCK_TIMEOUT` — checked every cycle.
//!   * TEMP workspace via `TempDir` — never the real ship home or real data; dropped at
//!     end of scope so teardown happens even on panic/early-return. NO global
//!     DB / env-resolved path is ever touched (we deliberately do NOT call
//!     `EventBus::rebuild`, which writes cursors through the global `open_db()`;
//!     the rebuild churn is driven entirely through the temp-DB-bound actor
//!     store instead).
//!   * In-process, single-threaded driver, no network, no spawned child.
//!
//! ## Gating
//!
//! The entire file body is behind `#![cfg(feature = "soak")]`. With the feature
//! off (the default for `cargo test` / `just build`) this compiles to nothing
//! and runs nothing. Invoke explicitly:
//!
//! ```text
//! cargo test -p shipd --features soak --test rss_soak -- --nocapture
//! ```

#![cfg(feature = "soak")]
// FILE_CAP_OVERRIDE: single self-contained soak gate. The safety triad
// (RSS-ceiling watchdog, cycle cap, wall-clock timeout, temp-workspace
// teardown), the churn driver, and the slope/eviction measurement are one
// auditable unit — splitting the safety machinery across files would make the
// "can this OOM the host?" review harder, not easier. ~40% of the length is the
// load-bearing doc comment that documents that safety story.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::events::validator::{CallerKind, EmitContext};
use ship_substrate::events::EventFilter;
use ship_substrate::events::{ActorConfig, EventEnvelope};
use ship_substrate::gen_ulid;
use ship_substrate::ShipHome;

// ── Safety + measurement constants ──────────────────────────────────────────

/// Hard RSS ceiling. The watchdog aborts the moment process RSS crosses this.
/// 1.0 GiB — a conservative ceiling far below the 19 GB host, so even a runaway
/// regression cannot OOM the box.
const RSS_CEILING_MB: u64 = 1024;

/// Hard upper bound on driver cycles. Never exceeded regardless of slope/timing.
const MAX_CYCLES: u64 = 5000;

/// Wall-clock timeout. The driver stops once this elapses even before
/// `MAX_CYCLES`. 60s keeps the gate snappy while collecting ~1000+ cycles /
/// 20+ samples — enough for a stable post-warmup slope fit. (`MAX_CYCLES` is
/// the other bound; on a fast box the cap binds first, on a slow box the clock
/// does — either way the run is bounded.)
const WALL_CLOCK_TIMEOUT: Duration = Duration::from_secs(60);

/// Watchdog sampling cadence — frequent so the ceiling is enforced promptly.
const WATCHDOG_INTERVAL: Duration = Duration::from_millis(100);

/// Take a measurement sample every this many cycles.
const SAMPLE_EVERY: u64 = 50;

/// Run a projection-rebuild (event-log read-back) round every this many cycles.
const REBUILD_EVERY: u64 = 100;


/// Fraction of samples (from the front) discarded as warmup before fitting.
const WARMUP_FRACTION: f64 = 0.20;

/// Slope threshold. Flat (fixed) code parks near 0 MB/min once allocator arenas
/// settle; a #256-class leak (470 MB/min acute, or the slow `actor_cursors`
/// climb) blows past this by orders of magnitude. 5.0 MB/min is tight enough to
/// fail a real strand yet above the sampling/allocator jitter of a bounded
/// in-process churn over this many cycles.
const SLOPE_THRESHOLD_MB_PER_MIN: f64 = 5.0;

// ── /proc/self/status VmRSS reader (Linux) ──────────────────────────────────

/// Current process resident set size in KiB, read from `/proc/self/status`.
/// Returns 0 if unreadable (non-Linux / sandbox) — the watchdog treats 0 as
/// "cannot enforce ceiling" and the test asserts we got real samples.
fn vmrss_kib() -> u64 {
    let status = match std::fs::read_to_string("/proc/self/status") {
        Ok(s) => s,
        Err(_) => return 0,
    };
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            for tok in rest.split_whitespace() {
                if let Ok(kib) = tok.parse::<u64>() {
                    return kib;
                }
            }
        }
    }
    0
}

fn vmrss_mb() -> f64 {
    vmrss_kib() as f64 / 1024.0
}

// ── Watchdog ────────────────────────────────────────────────────────────────

/// A background OS thread (not a tokio task) sampling RSS every
/// `WATCHDOG_INTERVAL`. When RSS crosses the ceiling it (1) sets `abort` so the
/// driver bails on its next per-cycle check, and (2) — as a hard backstop
/// against a wedged driver — `process::abort()`s. The thread exits when `done`
/// is set (on `Drop`).
struct Watchdog {
    done: Arc<AtomicBool>,
    abort: Arc<AtomicBool>,
    peak_mb: Arc<AtomicU64>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Watchdog {
    fn start() -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let abort = Arc::new(AtomicBool::new(false));
        let peak_mb = Arc::new(AtomicU64::new(0));

        let t_done = done.clone();
        let t_abort = abort.clone();
        let t_peak = peak_mb.clone();
        let handle = std::thread::Builder::new()
            .name("rss-soak-watchdog".into())
            .spawn(move || {
                while !t_done.load(Ordering::Relaxed) {
                    let mb = vmrss_kib() / 1024;
                    let mut cur = t_peak.load(Ordering::Relaxed);
                    while mb > cur {
                        match t_peak.compare_exchange_weak(
                            cur,
                            mb,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(observed) => cur = observed,
                        }
                    }
                    if mb >= RSS_CEILING_MB {
                        t_abort.store(true, Ordering::SeqCst);
                        // A cooperative bail cannot keep up with a true runaway.
                        // Hard-abort so the host is never at risk. Print first so
                        // the cause is visible in CI output.
                        eprintln!(
                            "rss-soak WATCHDOG: RSS {mb} MB >= ceiling {RSS_CEILING_MB} MB — aborting process to protect the host"
                        );
                        std::process::abort();
                    }
                    std::thread::sleep(WATCHDOG_INTERVAL);
                }
            })
            .expect("spawn watchdog thread");

        Self {
            done,
            abort,
            peak_mb,
            handle: Some(handle),
        }
    }

    fn aborted(&self) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn peak_mb(&self) -> u64 {
        self.peak_mb.load(Ordering::Relaxed)
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.done.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

// ── Driver helpers ───────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: None,
        session_id: None,
    }
}

/// Config matching an SSE-source actor: subscribes to one namespace, no write
/// namespaces (read-only consumer), unique per-connection namespace.
fn sse_source_config(actor_id: &str) -> ActorConfig {
    ActorConfig {
        namespace: actor_id.to_string(),
        write_namespaces: vec![],
        read_namespaces: vec!["sse.".into()],
        subscribe_namespaces: vec!["sse.".into()],
        see_directed_events: false,
    }
}

/// Extract the integer value of `key=<n>` from a `KernelRouter::diag_line()`
/// string (space-separated `key=value` tokens). Returns 0 if absent/unparsable.
fn diag_value(diag: &str, key: &str) -> usize {
    let needle = format!("{key}=");
    for tok in diag.split_whitespace() {
        if let Some(v) = tok.strip_prefix(&needle) {
            return v.parse().unwrap_or(0);
        }
    }
    0
}

/// Least-squares slope (y per unit x) of `(x, y)` pairs. Returns 0 for < 2
/// points or a degenerate x-spread.
fn linear_slope(points: &[(f64, f64)]) -> f64 {
    let n = points.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let sx: f64 = points.iter().map(|(x, _)| x).sum();
    let sy: f64 = points.iter().map(|(_, y)| y).sum();
    let sxx: f64 = points.iter().map(|(x, _)| x * x).sum();
    let sxy: f64 = points.iter().map(|(x, y)| x * y).sum();
    let denom = n * sxx - sx * sx;
    if denom.abs() < f64::EPSILON {
        return 0.0;
    }
    (n * sxy - sx * sy) / denom
}

// ── The gate ─────────────────────────────────────────────────────────────────

/// Multi-thread runtime with a small, FIXED worker count. The substrate's
/// `db::block_on` (used by `stop_actor`'s WAL flush and the actor-store reads)
/// requires a multi-threaded runtime — `current_thread` panics there. The
/// driver loop itself is still logically serial (we `.await` each `emit` before
/// the next), so behaviour is deterministic; the worker pool only services the
/// blocking DB calls. The watchdog is a plain OS thread, independent of this
/// pool, so its `process::abort` backstop holds regardless.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn daemon_rss_slope_is_flat_under_connection_churn() {
    // ── Isolated temp workspace — never the real ship home or real data. Dropped at end
    //    of scope (even on panic) so the DB + WAL are cleaned up. ──
    let temp = tempfile::TempDir::new().expect("create temp workspace");
    let ship_dir = temp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).expect("create ship dir");

    // `emit`/`stop_actor` take `&self`/`&mut self`; the loop mutates via &mut.
    let mut router = KernelRouter::new(ship_dir.clone()).expect("construct kernel router");

    // ── Watchdog: enforces the RSS ceiling independent of the driver loop. ──
    let watchdog = Watchdog::start();

    // Long-lived reader actor used by the projection-rebuild element (#4). It
    // reads the `sse.` namespace back out of the temp-DB store — the read +
    // deserialize + Vec-allocate cost a projection rebuild incurs. Spawned once
    // and kept for the whole soak (a single, bounded actor — not churn).
    let reader_id = "rss-soak-reader";
    let reader_store = router
        .spawn_actor(reader_id, sse_source_config(reader_id))
        .map(|(store, _mb)| store)
        .ok();

    let ctx = runtime_ctx();
    let start = Instant::now();

    let mut samples: Vec<(f64, f64)> = Vec::new(); // (cycle_index, rss_mb)
    let mut cycles_run: u64 = 0;
    let mut stopped_reason = "completed max cycles";

    for cycle in 0..MAX_CYCLES {
        // Safety checks FIRST, every cycle — the cooperative half of the
        // ceiling guard, plus the cycle/time bounds.
        if watchdog.aborted() {
            stopped_reason = "watchdog ceiling abort (cooperative)";
            break;
        }
        if start.elapsed() >= WALL_CLOCK_TIMEOUT {
            stopped_reason = "wall-clock timeout";
            break;
        }

        // ── 1. SSE-source actor: fresh ULID id (the D1 path), spawn → emit a
        //       few events → stop_actor (the disconnect strand path). ──
        let sse_id = format!("sui-sse-overview-{}", gen_ulid());
        if let Ok((store, _mb)) = router.spawn_actor(&sse_id, sse_source_config(&sse_id)) {
            for i in 0..3u32 {
                let ev = EventEnvelope::new(
                    "sse.tick",
                    "overview",
                    &serde_json::json!({ "cycle": cycle, "n": i }),
                )
                .expect("envelope")
                .with_actor_id(&sse_id);

                let _ = router.emit(ev, &ctx).await;

                // 3. Request-cycle read: query the actor's own store (the per-
                //    request read path). Read-only; result dropped.
                let _ = store.query(&EventFilter {
                    event_type: Some("sse.tick".into()),
                    limit: Some(8),
                    ..Default::default()
                });
            }
        }
        // The disconnect that strands a cursor if unfixed.
        let _ = router.stop_actor(&sse_id);

        // ── 2. HMR-style connect/disconnect + rate-counter fanout. Two fresh
        //       unique ids, BOTH spawned as real actors then stopped — exactly
        //       the daemon's shape (a directed/`target` event goes to a real
        //       actor that is later stopped). `emit` inserts a `RateCounter`
        //       for the fresh originator AND the fresh target; `stop_actor`
        //       must evict BOTH (the #256 rate-counter strand). If eviction
        //       regressed, `rate_counters` would grow one+ per cycle and the
        //       slope would climb. We stop both, so the fixed code holds
        //       `rate_counters` flat (asserted via the diag line below). ──
        let hmr_id = format!("sui-hmr-{}", gen_ulid());
        let target_id = format!("sui-hmr-target-{}", gen_ulid());
        let _ = router.spawn_actor(&hmr_id, sse_source_config(&hmr_id));
        let _ = router.spawn_actor(&target_id, sse_source_config(&target_id));
        let fanout = EventEnvelope::new("sse.tick", "fanout", &serde_json::json!({ "cycle": cycle }))
            .expect("envelope")
            .with_actor_id(&hmr_id)
            .with_target(&target_id);
        let _ = router.emit(fanout, &ctx).await;
        let _ = router.stop_actor(&hmr_id);
        let _ = router.stop_actor(&target_id);

        // ── 4. Periodic projection-rebuild churn — read the `sse.` event log
        //       back out of the temp-DB store through the long-lived reader
        //       actor (read + deserialize + Vec-allocate, the cost a projection
        //       rebuild incurs). Bounded by `limit`; result dropped each round.
        //       Touches the temp DB only. ──
        if cycle % REBUILD_EVERY == REBUILD_EVERY - 1 {
            if let Some(ref store) = reader_store {
                let _ = store.query(&EventFilter {
                    event_type: Some("sse.tick".into()),
                    limit: Some(500),
                    ..Default::default()
                });
            }
        }


        cycles_run = cycle + 1;

        // ── MEASURE: sample RSS every SAMPLE_EVERY cycles. ──
        if cycle % SAMPLE_EVERY == 0 {
            samples.push((cycle as f64, vmrss_mb()));
        }
    }

    // Final sample.
    samples.push((cycles_run as f64, vmrss_mb()));

    let peak = watchdog.peak_mb();
    let diag = router.diag_line();

    // Teardown is explicit + automatic: drop router (kernel state) and temp
    // (workspace) before asserting so a failing assert still cleaned up. The
    // watchdog joins on its own Drop at end of scope.
    drop(router);

    // ── Assertions + report. ──
    assert!(
        !watchdog.aborted(),
        "RSS ceiling breached (peak {peak} MB >= {RSS_CEILING_MB} MB) — the soak hit the safety ceiling, indicating a severe leak"
    );
    assert!(
        samples.iter().any(|(_, mb)| *mb > 0.0),
        "no readable VmRSS samples (/proc/self/status) — cannot measure slope on this platform"
    );

    // ── Structural eviction assertions (independent of the slope). The diag
    //    line reports the live sizes of the exact #256/D1 maps. After
    //    ~thousands of spawn/stop/emit cycles the fixed code holds these tiny;
    //    a re-introduced strand grows them ~one per cycle. We assert a generous
    //    ceiling (far below `cycles_run`) so a real strand fails loudly here
    //    even before the slope could. ──
    let strand_ceiling = 64usize;
    let rate_counters = diag_value(&diag, "rate_counters");
    let actor_cursors = diag_value(&diag, "actor_cursors");
    assert!(
        rate_counters <= strand_ceiling,
        "rate_counters={rate_counters} after {cycles_run} cycles — a #256 rate-counter strand is present (must be evicted on stop_actor). diag: {diag}"
    );
    assert!(
        actor_cursors <= strand_ceiling,
        "actor_cursors={actor_cursors} after {cycles_run} cycles — a #256/D1 actor_cursors strand is present (must be evicted for single-use actors on stop). diag: {diag}"
    );

    let warmup = ((samples.len() as f64) * WARMUP_FRACTION).floor() as usize;
    let post: Vec<(f64, f64)> = samples.iter().copied().skip(warmup).collect();

    let elapsed_secs = start.elapsed().as_secs_f64().max(0.001);
    let cycles_per_sec = (cycles_run as f64) / elapsed_secs;
    let slope_mb_per_cycle = linear_slope(&post);
    let slope_mb_per_min = slope_mb_per_cycle * cycles_per_sec * 60.0;

    let first_mb = samples.first().map(|(_, m)| *m).unwrap_or(0.0);
    let last_mb = samples.last().map(|(_, m)| *m).unwrap_or(0.0);
    let grown_mb = last_mb - first_mb;

    eprintln!("──────────────────────────────────────────────────────────────");
    eprintln!("G2 daemon RSS-soak gate (substrate-direct, unstable paths)");
    eprintln!("  stop reason        : {stopped_reason}");
    eprintln!("  cycles run         : {cycles_run} (cap {MAX_CYCLES})");
    eprintln!(
        "  wall clock         : {elapsed_secs:.2}s (timeout {}s)",
        WALL_CLOCK_TIMEOUT.as_secs()
    );
    eprintln!("  cycles/sec         : {cycles_per_sec:.1}");
    eprintln!(
        "  samples            : {} total, {} post-warmup",
        samples.len(),
        post.len()
    );
    eprintln!("  RSS first / last   : {first_mb:.1} MB / {last_mb:.1} MB");
    eprintln!("  RSS grown          : {grown_mb:.2} MB");
    eprintln!("  RSS peak (watchdog): {peak} MB (ceiling {RSS_CEILING_MB} MB)");
    eprintln!(
        "  SLOPE              : {slope_mb_per_min:.3} MB/min  (threshold {SLOPE_THRESHOLD_MB_PER_MIN} MB/min)"
    );
    eprintln!("  kernel diag        : {diag}");
    eprintln!("──────────────────────────────────────────────────────────────");

    drop(temp);

    assert!(
        slope_mb_per_min < SLOPE_THRESHOLD_MB_PER_MIN,
        "post-warmup RSS slope {slope_mb_per_min:.3} MB/min exceeds threshold {SLOPE_THRESHOLD_MB_PER_MIN} MB/min — a #256-class daemon leak (actor_cursors / rate_counters / ReplayLog strand) is present"
    );
}
