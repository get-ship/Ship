//! Phase 7 — server-render emission.
//!
//! Two paths:
//!
//! 1. **Rust function path (default for Ship)** — emits a Rust function
//!    the daemon dynamically loads + invokes per request. Typed,
//!    fastest. ~few µs per render call.
//!
//! 2. **Text-template fallback (OSS deployment)** — emits a Tera-like
//!    template the deployer's chosen engine can interpret without a
//!    Rust runtime. For static-export and Node-server scenarios.
//!
//! Selection via `CompileOpts::ssr_target`.
