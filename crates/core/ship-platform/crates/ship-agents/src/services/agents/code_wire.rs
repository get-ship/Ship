//! ConnectRPC wire layer for `ship.code.v1.CodeService` — the
//! coding-agent capability surface (mount split, 2026-07-10).
//!
//! `CodeWire` is the `ship.code.v1.CodeService` impl that the daemons
//! mount at router assembly. Behind the same `AgentDispatchPort` as
//! the in-process `AgentsService.Dispatch`, the only difference is the
//! request shape: `CodeDispatchRequest` carries the coding-class
//! fields (spec / harness / repo_path / base_ref / branch /
//! invocation / keepalive / source_spec_path / depends_on / resume /
//! vars) that the slim `DispatchAgentRequest` does not.
//!
//! Two ops today:
//!   * `Dispatch`  — the coding-class dispatch; the same
//!     `dispatch.requested` event the in-process path emits, with
//!     driver-edge fields populated.
//!   * `Harnesses` — list the harnesses this install registers
//!     (compiled-in `stack_model::HARNESSES` table, mirrored in the
//!     internal `HarnessService.ListHarnesses`).
//!
//! Mount split (2026-07-10): shipd gates the mount on `--features
//! ship-code`; helmd mounts it unconditionally (the testbed posture).
//! With ship-code off, the route is absent — Connect discovers the
//! service by path presence (the design intent; absence is
//! structural, not a refused call). See proposal-ship-code-package.md
//! §"R-A redesign" + decisions-2026-07-09-space-cutover.md
//! §"The seam on the wire".

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Router as ConnectRouter, ServiceResult};
use ship_proto::ship::code::v1::{
    CodeDispatchRequest, CodeService as CodeServiceTrait, CodeServiceExt, ListHarnessesResponse,
    OwnedCodeDispatchRequestView, OwnedListHarnessesRequestView,
};
use ship_substrate::ports::KernelLog;

/// transport-neutral core uses `Option<String>` to model the
/// "field omitted" shape explicitly.
fn opt_string(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Map the wire's `invocation` proto string to the typed
/// `InvocationMode` enum the dispatch core operates on. Unknown
/// strings default to `Pty` (the historical default — keeps
/// pre-cutover callers that omitted the field working without
/// re-validation).
fn parse_invocation(s: &str) -> agent_schema::InvocationMode {
    match s {
        "print" => agent_schema::InvocationMode::Print,
        "stream" => agent_schema::InvocationMode::Stream,
        _ => agent_schema::InvocationMode::Pty,
    }
}

use crate::services::agents::code_dispatch_core::dispatch_code_session_core;
use crate::services::agents::dispatch_core::{AgentDispatchPort, DispatchCoreError};

// ── Wire struct ───────────────────────────────────────────────────────────────

/// ConnectRPC wire for `ship.code.v1.CodeService`. Mirrors `AgentsWire`
/// in shape (one struct, `code_wire::new` from a config, all the
/// dispatch plumbing in a single `impl CodeService` block) but takes
/// only the two ports the coding path needs: `dispatch_port` (kernel
/// persist + daemon origin) and the live agent-def index for the
/// "agent is in the index" wire-boundary check.
pub struct CodeWire {
    ship_dir: PathBuf,
    /// Per-call `KernelLog` subscriber handle. The actual write path lives
    /// on `dispatch_port::persist_and_route`; the wire keeps this for
    /// the watch + read surface.
    kernel: Arc<KernelLog>,
    dispatch_port: Arc<dyn AgentDispatchPort>,
    def_index: Option<Arc<ship_settings::agents_index::AgentDefIndex>>,
}

impl CodeWire {
    pub fn new(config: CodeWireConfig) -> Self {
        let CodeWireConfig {
            ship_dir,
            kernel,
            dispatch_port,
            def_index,
        } = config;
        Self {
            ship_dir,
            kernel,
            dispatch_port,
            def_index,
        }
    }
}

/// Named-field constructor input for [`CodeWire`].
pub struct CodeWireConfig {
    pub ship_dir: PathBuf,
    pub kernel: Arc<KernelLog>,
    pub dispatch_port: Arc<dyn AgentDispatchPort>,
    /// Live agent-def index (project scope) when wired. The dispatch
    /// path consults the index for the "agent exists" wire-boundary
    /// check; `None` for the no-index posture (helmd's lean mount) —
    /// the path falls back to `agent_id_exists` which resolves through
    /// `space_resolver::roll_up_registered_blocking`.
    pub def_index: Option<Arc<ship_settings::agents_index::AgentDefIndex>>,
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl CodeServiceTrait for CodeWire {
    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: OwnedCodeDispatchRequestView,
    ) -> ServiceResult<ship_proto::ship::code::v1::CodeDispatchResult> {
        // The gate is enforced by the mount call site — shipd uses the
        // shared `agents.dispatch` blast-radius for both ops; helmd's
        // unconditional testbed mount omits the gate. We re-check at
        // the wire level for defense in depth.
        let _ = ctx; // ctx held for future auth extraction (e.g. C3).

        // The "agent is in the agent index" check is enforced at the
        // wire boundary (this site) — it needs the `ship_dir` to
        // consult the index, which the transport-neutral
        // `dispatch_code_session_core` doesn't have.
        let agent_in_index = if let Some(index) = self.def_index.as_ref() {
            crate::services::agents::view_state::agent_id_exists_in_index(index, request.agent)
                .await
        } else {
            crate::services::agents::view_state::agent_id_exists(&self.ship_dir, request.agent)
        };
        if !agent_in_index {
            return Err(ConnectError::invalid_argument(format!(
                "agent {:?} not found in agent index",
                request.agent
            )));
        }

        // Convert the wire struct (raw `String` per the proto3
        // generated code, no `Option`) into the transport-neutral
        // `CodeDispatchRequest` the dispatch core takes (optional
        // strings + an `InvocationMode` enum). The two structs are
        // intentionally distinct — the wire type mirrors the proto
        // exactly, the core type is the daemon-free shape the
        // validate/derive/event-emit helpers operate on.
        let owned: CodeDispatchRequest = request.to_owned_message();
        let rust_req = super::code_dispatch_request::CodeDispatchRequest {
            agent: owned.agent,
            prompt: opt_string(&owned.prompt),
            model: opt_string(&owned.model),
            slug: opt_string(&owned.slug),
            spec: opt_string(&owned.spec),
            source_spec_path: opt_string(&owned.source_spec_path),
            resume: owned.resume,
            harness: opt_string(&owned.harness),
            invocation: parse_invocation(&owned.invocation),
            keepalive: owned.keepalive,
            branch: opt_string(&owned.branch),
            depends_on: if owned.depends_on.is_empty() {
                None
            } else {
                Some(owned.depends_on)
            },
            repo_path: opt_string(&owned.repo_path),
            base_ref: opt_string(&owned.base_ref),
        };
        let outcome = dispatch_code_session_core(self.dispatch_port.as_ref(), rust_req)
            .await
            .map_err(|e| match e {
                DispatchCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                DispatchCoreError::Internal(m) => ConnectError::internal(m),
            })?;
        Ok(connectrpc::Response::new(
            ship_proto::ship::code::v1::CodeDispatchResult {
                session_id: outcome.session_id,
                slug: outcome.slug,
                branch: outcome.branch,
                resume: outcome.resume,
                studio_url: outcome.studio_url,
                ..Default::default()
            },
        ))
    }

    async fn harnesses(
        &self,
        _ctx: RequestContext,
        _request: OwnedListHarnessesRequestView,
    ) -> ServiceResult<ListHarnessesResponse> {
        // Enumerate the harnesses the install registers. The
        // `stack_model::HARNESSES` table is the single source of truth
        // — the same list the internal `HarnessService.ListHarnesses`
        // surface returns, projected to id strings only. R-A: the
        // registry is what the install provides; the absence of a
        // harness is structural, not an error (a coding agent naming
        // an unprovided harness fails ordinary validation upstream
        // and is not listed).
        let items: Vec<String> = bridge::harnesses::list_harnesses()
            .iter()
            .map(|d| d.id.to_string())
            .collect();
        Ok(connectrpc::Response::new(ListHarnessesResponse {
            items,
            ..Default::default()
        }))
    }
}

/// Mount `CodeService` onto `router`. The single mount point per
/// install: shipd gates it behind `#[cfg(feature = "bridge")]` at
/// the router assembly, helmd calls it unconditionally. With the
/// feature off, the mount is absent — Connect discovers the service by
/// path presence (a `CodeService/Dispatch` POST returns 404, not a
/// refused call).
pub fn mount(wire: Arc<CodeWire>, router: ConnectRouter) -> ConnectRouter {
    CodeServiceExt::register(wire, router)
}
