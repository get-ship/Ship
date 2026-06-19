//! Port traits for daemon-specific capabilities consumed by ship-ui domain logic.
//!
//! Follows the same pattern as `apps/shipd/src/agent_event_ports.rs` and
//! `pipeline_ports.rs`. ship-ui declares the trait; shipd implements it in
//! `apps/shipd/src/view_ports.rs` and injects the concrete adapter via the
//! `ViewWire` closures or direct state passing.
//!
//! # Traits
//!
//! - `ViewToolPort` — capabilities needed by `view_authoring::tool_dispatch`.
//!   Covers: bundler (compile-check), space-context construction (for snapshot
//!   ops), collections access, project slug, theme CSS resolution, and
//!   annotation SSE broadcast.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use ship_substrate::ports::SpaceContext;

// ── ViewToolPort ──────────────────────────────────────────────────────────────

/// Why a view preview could not be produced. Two variants because the wire
/// reports them differently — NotFound/view_not_found vs Internal/render_failed —
/// and the port must not flatten that distinction (verifier-caught fix).
#[derive(Debug, Clone)]
pub enum PreviewError {
    /// The view name resolved to nothing on disk.
    ViewNotFound(String),
    /// The view exists but rendering failed.
    RenderFailed(String),
}

/// Errors returned by bundler compile operations via `ViewToolPort`.
#[derive(Debug)]
pub enum BundlerBuildError {
    /// Build produced structured diagnostics.
    Bundle { message: String, file: Option<String>, line: Option<u32>, col: Option<u32> },
    /// Target name is not registered with the bundler.
    UnknownTarget(String),
    /// The sui-compiler binary is absent from the system.
    NoCompiler,
}

impl std::fmt::Display for BundlerBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle { message, .. } => write!(f, "{message}"),
            Self::UnknownTarget(t) => write!(f, "unknown bundler target: {t}"),
            Self::NoCompiler => write!(f, "sui-compiler unavailable — rebuild the daemon (`ship build`)"),
        }
    }
}

/// Capabilities the view-authoring tool dispatch needs from shipd.
#[async_trait]
pub trait ViewToolPort: Send + Sync {
    /// Compile a bundler target (e.g. `"views/{name}/view"`). Returns an
    /// opaque error on failure so the caller can extract diagnostic fields.
    async fn build_view(&self, target: &str) -> Result<(), BundlerBuildError>;

    /// Build a `SpaceContext` over the active project's data-store for `space`.
    /// Used by snapshot and list-snapshot operations.
    fn space_context(&self, space: &str) -> Arc<SpaceContext>;

    /// The daemon's `ship_dir` — base for artifact blob paths.
    fn ship_dir(&self) -> PathBuf;

    /// `CollectionService` over the active space. Returns `Err` when no active
    /// space is set on the daemon.
    fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService>;

    /// Active project slug (used as `space_id` for snapshot scoping).
    fn project_slug(&self) -> Option<String>;

    /// Resolved active-theme CSS — parses custom properties for `theme_tokens`.
    fn resolve_theme_css(&self) -> String;

    /// Resolve a view name to its (source content, on-disk path).
    /// Used by read_source, write_source, and preview_snapshot ops.
    fn resolve_view_source(&self, name: &str) -> Result<(String, std::path::PathBuf), String>;

    /// Compile a view by name to HTML for the preview_snapshot op.
    fn preview_view_html(&self, name: &str) -> Result<String, PreviewError>;
}
