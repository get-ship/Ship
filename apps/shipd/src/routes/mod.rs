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

pub mod actor_api;
#[cfg(test)]
mod actor_api_tests;
pub mod actors_api;
pub mod artifacts_api;
#[cfg(test)]
mod artifacts_api_tests;
#[cfg(feature = "ship-code")]
pub mod hooks_api;
pub mod human_webhook;
pub mod runtime_api;
pub mod sandbox_api;
#[cfg(test)]
mod sandbox_api_tests;
pub mod schema_api;
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
    // First-run `/` + `/setup` are owned by `deck::router()` now — shipd
    // serves no frontend surface, so those route variants were removed.

    // ── Session / auth ───────────────────────────────────────────────────

    /// @route POST "/api/v1/session"
    /// @returns SessionIssueResponse
    /// @auth scope Full
    IssueSession,

    // Account ops are served by the typed Connect `AccountService`
    // (account.proto, `ship-account` crate) — there is no REST account surface.

    // Shared CSS/theme assets (`view-tokens.css`, `theme.css`, `/api/v1/themes`),
    // the dev HMR firehose (`/_deck/dev/hmr`), and the per-tag source viewer
    // (`/_deck/source/{tag}`) are all owned by `deck::router()` now — shipd
    // serves no frontend surface, so those route variants were removed.

    // Pipeline dispatch moved to the typed Connect wire
    // (`PipelinesService.Dispatch`, ship.ops.v1) — the REST `POST
    // /api/v1/pipelines/run` route had no handler and only generated a dead
    // TS surface. The CLI (`ship_run::dispatch_pipeline`) calls the wire.

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


    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/daemon/status"
    /// @auth scope ReadWorkspace
    DaemonStatus,

    // ── Events / projections ─────────────────────────────────────────────
    // The SSE event stream (`GET /api/v1/events/stream`) was retired: the
    // kernel event bus is served once, over Connect `StreamService.Events`
    // (ship.ops.v1). Web + SDK consumers tail that stream.

    // POST /api/v1/rebuild-projections was removed: no caller
    // (admin-only vestige) — the kernel can self-heal projections
    // on demand via the existing event log. The rebuild primitive
    // lives at `ship_substrate::db::rebuild::rebuild_projections`
    // and is still wired into `ShipRuntime::init` for boot-time
    // projection recovery.

    /// @route GET "/api/v1/projects/{id}/pty"
    /// @auth scope ManageRuns
    ProjectPty { id: String },

    // Workspace-scoped artifact status/promote removed — callers get 404.
    // Use the unified /api/artifacts/* surface instead.

    // ── Actor kernel (L0 — kernel-directed mailbox delivery) ─────────────
    // The mesh.* introspection reads (GET /api/v1/mesh, per-actor reads)
    // were culled with the mesh op surface, and POST /api/v1/events/route
    // (the directed-event HTTP emit) was culled with zero consumers —
    // events flow through the kernel's own emit paths. actor/spawn stays:
    // it is the register-or-refine seam for externally-launched agents
    // (SHIP_ACTOR_ID) and is exercised by tests/loops/daemon-minted-ids.

    /// @route POST "/api/v1/actor/spawn"
    /// @auth scope MeshSend
    ActorSpawn,

    /// @route GET "/api/v1/actors"
    /// @auth scope ReadWorkspace
    ListActors,

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

    /// @route HEAD "/api/v1/artifacts/{id}"
    /// @auth handler
    ArtifactHead { id: String },

    /// @route GET "/api/v1/artifacts/{id}/metadata"
    /// @auth handler
    ArtifactMetadata { id: String },

    // GET /api/v1/capabilities was removed: zero consumers (only shipd's own
    // tests fetched it) and it served fabricated tier/feature data naming
    // retired concepts (jobs, workspaces).

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
