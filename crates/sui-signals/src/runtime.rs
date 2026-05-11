//! Thread-local reactive tracking state.
//!
//! All reactive primitives (signal reads, effect runs, derived recomputes)
//! are synchronous. Thread-locals are correct here: the tracking scope never
//! crosses an await point; each thread maintains its own observer stack.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// A subscriber callback held alive by the owner (Effect/Derived).
/// Signals hold `Weak<dyn Fn()>` references to avoid keeping dead subscribers alive.
pub(crate) type SubFn = Arc<dyn Fn() + Send + Sync>;

thread_local! {
    /// The currently-executing observer (effect run fn or derived invalidate fn).
    /// When Some, any `Signal::get()` call auto-subscribes this observer.
    pub(crate) static CURRENT_OBSERVER: RefCell<Option<SubFn>> = const { RefCell::new(None) };

    /// Nesting depth of active `batch()` calls. Subscribers are deferred while > 0.
    pub(crate) static BATCH_DEPTH: Cell<u32> = const { Cell::new(0) };

    /// Subscribers queued for flush when the outermost `batch()` returns.
    pub(crate) static PENDING: RefCell<Vec<SubFn>> = const { RefCell::new(Vec::new()) };
}

/// Swap the current observer. Returns the previous one.
pub(crate) fn set_observer(next: Option<SubFn>) -> Option<SubFn> {
    CURRENT_OBSERVER.with(|obs| {
        let prev = obs.borrow().clone();
        *obs.borrow_mut() = next;
        prev
    })
}

/// Clone the current observer without consuming it.
pub(crate) fn get_observer() -> Option<SubFn> {
    CURRENT_OBSERVER.with(|obs| obs.borrow().clone())
}

/// True while inside a `batch()` call.
pub(crate) fn is_batching() -> bool {
    BATCH_DEPTH.with(|d| d.get() > 0)
}

pub(crate) fn batch_enter() {
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));
}

/// Decrement batch depth. If reaching zero, flush all pending subscribers.
pub(crate) fn batch_exit() {
    BATCH_DEPTH.with(|d| {
        let depth = d.get();
        if depth > 0 {
            d.set(depth - 1);
        }
    });
    if !is_batching() {
        flush_pending();
    }
}

pub(crate) fn queue_pending(subs: Vec<SubFn>) {
    PENDING.with(|p| {
        let mut pending = p.borrow_mut();
        for sub in subs {
            // Deduplicate by pointer identity, matching TS `Set<Subscriber>` semantics.
            if !pending.iter().any(|s| Arc::ptr_eq(s, &sub)) {
                pending.push(sub);
            }
        }
    });
}

fn flush_pending() {
    let subs: Vec<SubFn> = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
    for s in subs {
        s();
    }
}
