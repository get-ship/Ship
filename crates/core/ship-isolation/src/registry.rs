//! Maps `IsolationStrategy` (the symbolic identity) to a concrete
//! `Isolator` instance.
//!
//! Pluggable: the daemon constructs a `Registry` at startup, registers
//! per-strategy factories (each gated by the relevant Cargo feature),
//! and hands the registry to the orchestrator. The orchestrator calls
//! `registry.build(resolved.strategy, &cfg)` to materialize an
//! `Isolator` for one sandbox launch.
//!
//! Keeping this here (instead of in `ship-platform`) means
//! `ship-isolation` owns the full set of strategy aliases + the binding
//! convention. The daemon only registers things; it doesn't reinvent
//! the lookup.

use std::collections::HashMap;
use std::sync::Arc;

use crate::backends;
use crate::tier::IsolationStrategy;
use ship_substrate::isolation::Isolator;

/// A factory closure that constructs an `Isolator` for one launch.
/// Returns `Arc` so the registry can hand the same instance to many
/// concurrent sandboxes (most backends are stateless aside from per-
/// sandbox state stashed inside the `SandboxHandle`).
pub type IsolatorFactory = Arc<dyn Fn() -> Arc<dyn Isolator> + Send + Sync>;

/// Registry of strategies the daemon can materialize. Built once at
/// shipd startup; immutable after.
#[derive(Clone, Default)]
pub struct Registry {
    inner: HashMap<IsolationStrategy, IsolatorFactory>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a strategy. Last call wins for a given key — callers
    /// register defaults first, then layer user-supplied overrides.
    pub fn register<F>(&mut self, strategy: IsolationStrategy, factory: F)
    where
        F: Fn() -> Arc<dyn Isolator> + Send + Sync + 'static,
    {
        self.inner.insert(strategy, Arc::new(factory));
    }

    /// Build an `Isolator` for one launch. Returns `None` when the
    /// strategy has no factory registered — the orchestrator should
    /// treat this as `RecommenderError::StrategyUnavailable` for that
    /// caller's reporting purposes.
    pub fn build(&self, strategy: IsolationStrategy) -> Option<Arc<dyn Isolator>> {
        self.inner.get(&strategy).map(|f| f())
    }

    pub fn registered(&self) -> impl Iterator<Item = IsolationStrategy> + '_ {
        self.inner.keys().copied()
    }
}

/// Build a registry pre-populated with the always-available backends
/// (currently just `Noop`). Feature-gated backends (DockerEngine,
/// YoukiRootless, sidecar, etc.) register themselves via
/// `register_default_<name>()` helpers below when their crate features
/// are enabled.
pub fn default_registry() -> Registry {
    let mut r = Registry::new();
    register_default_noop(&mut r);
    r
}

pub fn register_default_noop(r: &mut Registry) {
    r.register(IsolationStrategy::Noop, || Arc::new(backends::Noop));
}

/// Convenience: register the Docker backend with a config the daemon
/// pre-resolves. Caller-built so the same registry can target docker
/// in one workspace and podman in another.
pub fn register_docker(r: &mut Registry, cfg: backends::DockerConfig) {
    r.register(IsolationStrategy::DockerEngine, move || {
        Arc::new(backends::Docker::new(cfg.clone()))
    });
}

/// Convenience: register the HarnessSandbox strategy (which maps to the
/// `SandboxRuntime` backend wrapping the Anthropic `srt` package).
pub fn register_harness_sandbox(r: &mut Registry, cfg: backends::SandboxRuntimeConfig) {
    r.register(IsolationStrategy::HarnessSandbox, move || {
        Arc::new(backends::SandboxRuntime::new(cfg.clone()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::isolation::SandboxHandle;
    use ship_substrate::isolation::SandboxPolicy;
    use ship_substrate::isolation::IsolatorError;
    use async_trait::async_trait;

    struct DummyIsolator(&'static str);

    #[async_trait]
    impl Isolator for DummyIsolator {
        fn name(&self) -> &'static str {
            self.0
        }
        async fn available(&self) -> Result<(), IsolatorError> {
            Ok(())
        }
        async fn prepare(
            &self,
            _policy: &SandboxPolicy,
        ) -> Result<SandboxHandle, IsolatorError> {
            unreachable!()
        }
        async fn shutdown(&self, _sandbox: SandboxHandle) -> Result<(), IsolatorError> {
            Ok(())
        }
    }

    #[test]
    fn default_registry_has_noop() {
        let r = default_registry();
        let iso = r.build(IsolationStrategy::Noop).expect("noop registered");
        assert_eq!(iso.name(), "noop");
    }

    #[test]
    fn missing_strategy_returns_none() {
        let r = default_registry();
        assert!(r.build(IsolationStrategy::DockerEngine).is_none());
    }

    #[test]
    fn last_registration_wins() {
        let mut r = Registry::new();
        r.register(IsolationStrategy::Noop, || Arc::new(DummyIsolator("first")));
        r.register(IsolationStrategy::Noop, || Arc::new(DummyIsolator("second")));
        assert_eq!(r.build(IsolationStrategy::Noop).unwrap().name(), "second");
    }

    #[test]
    fn register_docker_round_trips() {
        let mut r = default_registry();
        register_docker(&mut r, backends::DockerConfig::default());
        let iso = r.build(IsolationStrategy::DockerEngine).unwrap();
        assert_eq!(iso.name(), "docker");
    }
}
