//! The shipd-side adapters for deck-core's `DataPort` and `ChangeSource`.
//!
//! These are the SHIP half of the multiplex transport move (#276): deck-core
//! owns the wire + state mechanics, shipd supplies:
//!   * `ShipDataPort` — implements `DataPort::resolve` by delegating to
//!     `ApiState::live_resolvers()`. The host's state (the `ApiState` clone)
//!     is bound INSIDE the impl, so it never crosses the deck boundary.
//!   * `ShipChangeSource` — implements `ChangeSource::open_feed` by spawning
//!     ONE kernel actor per stream (subscribed to the `config` namespace),
//!     the same body that lived in `window_subscription_rpc.rs:156-170`. The
//!     returned `ShipChangeFeed` forwards mailbox recvs as change ticks;
//!     dropping it tears down the actor via the RAII `ActorGuard` (the
//!     kernel-config pattern from `live_read::live_read_inner`).
//!
//! Constructed at router-mount time in `apps/shipd/src/router.rs` and
//! injected into `deck_axum::multiplex_router()` as a single
//! `Extension<MultiplexState>` carrying all three pieces.

use std::future::Future;
use std::pin::Pin;

use deck_core::{ChangeFeed, ChangeSource, DataPort};
use serde_json::Value;
use ship_substrate::events::{ActorConfig, Mailbox};

use crate::ship_runtime::{ActorGuard, ApiState};

/// shipd's `DataPort` impl — binds `ApiState` so deck-core never names it.
pub struct ShipDataPort {
    state: ApiState,
}

impl ShipDataPort {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

impl DataPort for ShipDataPort {
    fn resolve(&self, domain: &str, op: &str, args: &Value) -> Option<Value> {
        // LiveResolverRegistry::resolve_fn binds args, returns the args-free
        // ResolveFn. We then call it against the bound state to get the value.
        // None surfaces to deck as an explicit error (the wire never fabricates
        // a `null` initial value for an unregistered op).
        let resolve_fn = self.state.live_resolvers().resolve_fn(domain, op, args)?;
        Some(resolve_fn(&self.state))
    }

    fn is_provisioned(&self) -> bool {
        // The host is provisioned once at least one active workspace exists —
        // the exact check the retired `setup.rs::has_any_workspace()` used.
        // Drives deck's first-run routing (serve_index / serve_setup).
        ship_substrate::db::projects::list_active()
            .map(|p| !p.is_empty())
            .unwrap_or(false)
    }
}

/// shipd's `ChangeSource` impl — opens a kernel `config` subscription per
/// multiplex stream.
pub struct ShipChangeSource {
    state: ApiState,
}

impl ShipChangeSource {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

impl ChangeSource for ShipChangeSource {
    fn open_feed(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn ChangeFeed>, String>> + Send + '_>> {
        let state = self.state.clone();
        Box::pin(async move {
            // ONE kernel actor per multiplex stream. Subscribes the namespaces
            // whose events should re-resolve every live sub: `config` (settings,
            // collections, etc.) AND `agent` — so editing an agent file
            // (`agent.def.updated`, the `agent` namespace) makes the live agents
            // surface update instead of going stale until an unrelated
            // `config.changed`. Mirrors `live_read::live_read_inner:35-49`.
            let actor_id = format!("mux-{}", ship_substrate::gen_ulid());
            let config = ActorConfig {
                namespace: actor_id.clone(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec!["config".to_string(), "agent".to_string()],
                see_directed_events: false,
            };
            let mailbox = {
                let mut kernel = state.kernel().lock().await;
                kernel
                    .spawn_actor(&actor_id, config)
                    .map(|(_store, mb)| mb)
                    .map_err(|e| format!("spawn multiplex actor: {e}"))?
            };
            let guard = ActorGuard {
                actor_id: actor_id.clone(),
                kernel: state.kernel().clone(),
            };
            Ok(Box::new(ShipChangeFeed { mailbox, _guard: Some(guard) })
                as Box<dyn ChangeFeed>)
        })
    }
}

/// One open kernel `config` subscription, projected to deck's payload-free
/// `ChangeFeed`. The `ActorGuard` field is private; dropping the feed (on
/// stream end / client disconnect / reaper) drops the guard, which calls
/// `kernel.stop_actor` and tears down the kernel mailbox.
struct ShipChangeFeed {
    mailbox: Mailbox,
    /// RAII teardown: when the feed is dropped, the actor is stopped.
    /// Wrapped in `Option` so the drop impl can `.take()` it.
    _guard: Option<ActorGuard>,
}

impl ChangeFeed for ShipChangeFeed {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + Send + '_>> {
        Box::pin(async move {
            // The kernel's mailbox returns `Option<EventEnvelope>` — we
            // collapse it to deck's payload-free `Option<()>`: `None` means
            // the feed ended (deck closes the stream); `Some` means "a
            // config event occurred, re-resolve every sub".
            self.mailbox.recv().await.map(|_ev| ())
        })
    }
}

impl Drop for ShipChangeFeed {
    fn drop(&mut self) {
        // The guard's own Drop impl stops the actor; we take it out of the
        // Option so the field is consumed here (vs. at struct drop time
        // when `_guard` may already have been moved).
        let _ = self._guard.take();
    }
}
