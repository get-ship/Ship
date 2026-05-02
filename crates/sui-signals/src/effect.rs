//! `effect(fn) -> Disposer` — reactive side effects.

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use crate::runtime::{SubFn, set_observer};

struct EffectCore {
    active: AtomicBool,
    /// Self-reference used to pass this effect as the current observer during runs.
    /// Initialized once immediately after construction; always Some when `run` executes.
    run_ref: OnceLock<SubFn>,
}

/// Handle returned by `effect`. Severs the subscription and stops re-runs when dropped.
pub struct Disposer(Arc<EffectCore>);

impl Drop for Disposer {
    fn drop(&mut self) {
        self.0.active.store(false, Ordering::Release);
    }
}

/// Run `f` once eagerly to establish subscriptions, then re-run `f` whenever
/// any signal read inside `f` changes. Returns a `Disposer` that stops further
/// runs when dropped.
///
/// Re-runs are synchronous: they happen on the thread that calls `Signal::set`,
/// inside the signal's notification loop.
pub fn effect<F: Fn() + Send + Sync + 'static>(f: F) -> Disposer {
    let core = Arc::new(EffectCore {
        active: AtomicBool::new(true),
        run_ref: OnceLock::new(),
    });

    let core_weak = Arc::downgrade(&core);
    let f = Arc::new(f);

    let run: SubFn = Arc::new({
        let core_weak = core_weak.clone();
        let f = Arc::clone(&f);
        move || {
            let Some(core) = core_weak.upgrade() else { return };
            if !core.active.load(Ordering::Acquire) {
                return;
            }
            let Some(run_fn) = core.run_ref.get() else { return };
            let prev = set_observer(Some(Arc::clone(run_fn)));
            f();
            set_observer(prev);
        }
    });

    // Store the run Arc inside core so the closure can clone it as CURRENT_OBSERVER.
    // `run` outlives this store (it's also held locally), so the OnceLock never sees None.
    core.run_ref
        .set(Arc::clone(&run))
        .unwrap_or_else(|_| unreachable!("OnceLock set exactly once"));

    // Eager initial run establishes subscriptions.
    run();

    Disposer(core)
}
