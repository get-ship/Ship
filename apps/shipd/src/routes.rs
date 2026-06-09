//! Canonical Ship route registry.
//!
//! Parsed at build time by `apps/shipd/build.rs` to emit two artifacts:
//!
//! - `$OUT_DIR/routes_generated.rs` — Rust URL builders (`Route::method()`,
//!   `Route::full_path()`) and the `ShipRouterExt::ship_route(Route, handler)`
//!   trait extension used by `lib.rs` to register routes.
//! - `packages/deck/sdk/src/generated/routes.ts` — typed TypeScript route
//!   catalog for SDK / web consumers. Relative URLs (cookie-authenticated).
//!
//! # Variant syntax
//!
//! Each variant MUST carry a `@route` doc annotation. Optional annotations
//! describe the typed request/response surface and are used for codegen
//! only — Axum's own `Handler` trait enforces the Rust handler signature at
//! the `ship_route(Route::X, handler)` call site.
//!
//! ```ignore
//! /// @route METHOD "/path/with/{params}"
//! /// @returns Vec<PipelineRun>     (optional — TS gets typed fetch wrapper)
//! /// @body   CreateJobRequest    (optional — TS gets typed request body)
//! ```
//!
//! Path params (`{id}`) MUST match a String-typed field on the variant.
//! `build.rs` panics with a clear message if these drift.
//!
//! Type allowlist for `@returns` / `@body`: plain identifier (`Spec`),
//! `Vec<T>`, `Option<T>`, where T is a plain identifier. Anything else is a
//! build-time error.

#![allow(dead_code)] // Variants without handlers are still used by codegen.

// ── Route handler submodules ────────────────────────────────────────────
//
// Handler files live at `routes/<x>_api.rs`. Callers import them as
// `crate::routes::<x>_api::*` (or `shipd::routes::<x>_api::*` externally).

pub mod agent_sessions_attach;
#[cfg(test)]
mod agent_sessions_attach_tests;
pub mod artifacts_api;
pub mod hooks_api;
pub mod human_webhook;
pub mod rest_api;
pub mod runtime_api;
pub mod sandbox_api;
#[cfg(test)]
mod sandbox_api_tests;
#[cfg(test)]
mod scope_enforcement_tests;
pub mod webhook_ext;

// The build script writes generated impls into OUT_DIR. We include them here
// so `Route::full_path()`, `Route::method()`, and `ShipRouterExt` are
// visible to the rest of the crate.
include!(concat!(env!("OUT_DIR"), "/routes_generated.rs"));

/// Canonical list of every HTTP route registered by the daemon.
///
/// Variants with path params must keep those params as `String` fields so
/// URL builders can substitute them. Variants without params are unit
/// variants.
pub enum Route {
    // ── First-run / setup ────────────────────────────────────────────────

    /// @route GET "/"
    /// @auth public
    Index,

    /// @route GET "/setup"
    /// @auth public
    Setup,

    // ── Session / auth ───────────────────────────────────────────────────

    /// @route POST "/api/v1/session"
    /// @returns SessionIssueResponse
    /// @auth scope Full
    IssueSession,

    // Account ops are served by the typed Connect `AccountService`
    // (account.proto, `ship-account` crate) — there is no REST account surface.

    // ── Shared assets (unauthenticated) ──────────────────────────────────

    /// @route GET "/api/assets/view-tokens.css"
    /// @auth public
    AssetViewTokensCss,

    /// @route GET "/api/assets/theme.css"
    /// @auth public
    AssetThemeCss,

    /// @route GET "/api/v1/themes"
    /// @auth public
    ListThemes,

    // SUI framework hosting (`runtime.js` / `dom.js` / `sdk-client.js` /
    // `live-reload.js` / `components.js`) lives entirely in deck-axum's
    // `deck::router()` — see `crates/deck/deck-core/src/routes.rs::DECK_PREFIX`.
    // shipd only owns the two handler-style endpoints: HMR SSE + per-tag
    // source viewer, both renamed `/_deck/*` → `/_deck/*` in #277b.

    /// @route GET "/_deck/dev/hmr"
    /// @auth public
    SuiDevHmr,

    /// @route GET "/_deck/source/{tag}"
    /// @auth public
    SuiSource { tag: String },

    // ── PipelineService — service-shaped CRUD + run ops ──

    /// @route POST "/api/v1/pipelines/run"
    /// @auth scope CreateRuns
    PipelinesRun,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/v1/ship/schema"
    /// @auth scope ReadWorkspace
    ShipSchema,

    /// @route POST "/api/v1/sessions/spawn"
    /// @auth scope ManageRuns
    SpawnPtySession,

    /// @route GET "/api/v1/pty-sessions"
    /// @auth scope ReadRuns
    ListPtySessions,

    /// @route GET "/api/v1/sessions/{id}"
    /// @auth scope ReadRuns
    GetPtySession { id: String },

    /// @route DELETE "/api/v1/sessions/{id}"
    /// @auth scope ManageRuns
    StopPtySession { id: String },

    /// @route POST "/api/v1/sessions/{id}/input"
    /// @auth scope ManageRuns
    PtySessionInput { id: String },

    /// @route POST "/api/v1/sessions/{id}/resize"
    /// @auth scope ManageRuns
    PtySessionResize { id: String },

    /// @route POST "/api/v1/sessions/{id}/message"
    /// @auth scope ManageRuns
    PtySessionMessage { id: String },

    /// @route POST "/api/v1/sessions/{id}/reply"
    /// @auth scope ManageRuns
    PtySessionReply { id: String },

    /// @route GET "/api/v1/sessions/{id}/stream"
    /// @auth scope ManageRuns
    StreamPtySession { id: String },

    /// @route GET "/api/v1/sessions/{id}/events"
    /// @auth scope ReadRuns
    StreamPtySessionEvents { id: String },

    /// @route GET "/api/v1/harnesses/installed"
    /// @auth scope ReadWorkspace
    InstalledProviders,

    /// @route GET "/api/v1/sessions/active-across-projects"
    /// @auth scope ReadWorkspace
    ListSessionsAcrossProjects,

    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/daemon/status"
    /// @auth scope ReadWorkspace
    DaemonStatus,

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/v1/events/stream"
    /// @auth scope ReadWorkspace
    EventStream,

    /// @route POST "/api/v1/rebuild-projections"
    /// @auth scope AdminDaemon
    RebuildProjections,

    /// @route GET "/api/v1/projects/{id}/pty"
    /// @auth scope ManageRuns
    ProjectPty { id: String },

    // Workspace-scoped artifact status/promote removed — callers get 404.
    // Use the unified /api/artifacts/* surface instead.

    // ── Artifacts ────────────────────────────────────────────────────────
    // The list/upload/download/metadata/patch/delete handlers compute the
    // required scope dynamically from the artifact's stored `scope` field
    // ("run" / "project" / "user") via can_*_artifacts. The router can't
    // model that statically, so these declare `@auth handler` — the
    // scope_enforcement_tests assert that each handler still calls
    // can_*_artifacts (or require_scope) so the Handler tag can't silently
    // disable enforcement.

    /// @route POST "/api/v1/artifacts"
    /// @auth handler
    ArtifactsUpload,

    /// @route GET "/api/v1/artifacts/{id}"
    /// @auth handler
    ArtifactDownload { id: String },


    /// @route GET "/api/v1/artifacts/{id}/metadata"
    /// @auth handler
    ArtifactMetadata { id: String },

    // ── Capabilities ─────────────────────────────────────────────────────

    /// @route GET "/api/v1/capabilities"
    /// @auth scope ReadWorkspace
    Capabilities,

    // Isolation reads moved to the proto IsolationService (ship.ops.v1);
    // the REST routes were removed to avoid a parallel path.

    /// @route GET "/api/v1/sandboxes"
    /// @auth scope ReadWorkspace
    SandboxList,

    /// @route GET "/api/v1/sandboxes/{id}"
    /// @auth scope ReadWorkspace
    SandboxGet { id: String },
}

#[cfg(test)]
mod tests;
