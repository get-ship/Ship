// FILE_CAP_OVERRIDE: the write-side dispatch chain has one fall-through arm per
// command domain (intrinsically long, and splitting it hides the single
// enforcement point), plus the gate's security test suite lives beside the gate
// it exercises. The domain handlers are already separate submodules; what
// remains here is the one `call` gate + dispatch + its proofs.
//! Write-side command registry — the mutation analog of `services::registry`.
//!
//! `call(name, args, ctx)` dispatches a named command (declared in
//! `schema::COMMANDS`) to its domain handler, which builds + emits events or
//! delegates to a platform service. REST / Connect / MCP are thin projections
//! over this, and `cargo xtask gen-ops` generates the Connect
//! `CommandOpsService` from `COMMANDS` — one source of truth for the write
//! surface, exactly as `OPS` + `registry::call` are for reads.
//!
//! Logic currently inline in route handlers is lifted into the domain
//! submodules here (and, where it's genuine domain logic, on into
//! ship-platform services) per #78. Each lift makes one more transport thin.

use serde_json::Value;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;

mod actor;
mod agent;
mod agents;
mod annotations;
mod git;
mod mesh;
mod pipelines;
mod settings;
mod skills;
mod pipeline;
mod project;
mod surfaces;
mod views;

/// Ambient context for a command: the runtime + the authenticated caller
/// (identity + emit context). The write-side analog of passing `&ApiState` to
/// a read.
pub struct CommandCtx<'a> {
    pub state: &'a ApiState,
    pub auth: &'a AuthContext,
}

/// Structured command error carrying the HTTP status each transport maps to its
/// own wire shape (REST status code, Connect error code).
#[derive(Debug, Clone)]
pub struct CommandError {
    pub status: axum::http::StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn bad_args(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::BAD_REQUEST,
            code: "bad_args",
            message: msg.into(),
        }
    }
    pub fn unknown_command(name: &str) -> Self {
        Self {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "unknown_command",
            message: format!("unknown command `{name}`"),
        }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "not_found",
            message: msg.into(),
        }
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::FORBIDDEN,
            code: "forbidden",
            message: msg.into(),
        }
    }
    pub fn service(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            code: "service_error",
            message: msg.into(),
        }
    }
    /// Convert a route-core `(StatusCode, String)` error into a `CommandError`.
    pub fn from_core((status, message): (axum::http::StatusCode, String)) -> Self {
        let code = if status.is_client_error() { "bad_args" } else { "service_error" };
        Self { status, code, message }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
impl std::error::Error for CommandError {}

/// Dispatch a named command. Each domain submodule's `try_dispatch` returns
/// `Ok(Some(value))` when it owns `name`, `Ok(None)` to fall through.
pub async fn call(name: &str, args: &Value, ctx: &CommandCtx<'_>) -> Result<Value, CommandError> {
    // The one capability gate (spec: .artifacts/auth-gate-synthesis.md). Every
    // write surface — REST and Connect — reaches commands through here, so this
    // is the single enforcement point.
    //
    // Blast radius is resolved [`blast_for`]: proto first — the single source
    // (#67 a), read from the compiled FileDescriptorSet via `op_policy` keyed by
    // the canonical op name — then the hand-authored `COMMANDS.blast_radius` for
    // commands not yet on a per-domain proto service (`actor.*`, `mesh.*`,
    // `project.*`, `create.surface`, …). Either way a *known* command always
    // resolves to a tier and is gated.
    //
    // FAIL CLOSED (#314): the previous gate ran `if let Some(pol) =
    // op_policy::policy_for(name)` and SKIPPED on `None` (fail OPEN). After #78
    // deleted the PascalCase op-monolith proto, the old op-name lookup matched
    // nothing, so every command — including C2 — fell through ungated. Now an
    // unresolved *known* command is DENIED; only a truly unknown name (not a
    // command) falls through to the dispatch chain's `unknown_command` (404).
    {
        use crate::services::gate;
        if is_known_command(name) {
            let blast = match blast_for(name) {
                Some(b) => b,
                None => {
                    // Known command, no resolvable blast anywhere: deny rather
                    // than run it ungated. A registered write op must always gate.
                    tracing::error!(
                        target: "ship::security::audit",
                        op = %name,
                        "known command has no resolvable blast radius — DENIED (fail closed, #314)"
                    );
                    return Err(CommandError::forbidden(format!(
                        "command `{name}` has no gate policy — denied (fail closed)"
                    )));
                }
            };
            let service = gate::service_of(name);
            let caller = gate::caller_from_auth(ctx.auth);
            if let gate::Decision::Deny(msg) = gate::decide(blast, service, &caller) {
                return Err(CommandError::forbidden(msg));
            }
            // Security observability (S6, #124): the operator cred clears C2/C3
            // (credential mint, billing, acting as another actor). Operator
            // authority derives SOLELY from a verified, pid-attested operator
            // cred — the CLI transport and on-disk daemon token no longer
            // elevate. Record every such clearance as the audit trail of what a
            // verified operator cred reaches. Non-breaking — log only.
            if caller.is_operator
                && matches!(
                    blast,
                    gate::BlastRadius::C2Elevated | gate::BlastRadius::C3Critical
                )
            {
                tracing::warn!(
                    target: "ship::security::audit",
                    op = %name,
                    blast = blast.as_descriptor(),
                    agent_id = ?ctx.auth.agent_id,
                    "elevated op (C2/C3) cleared by a verified operator credential (S6 #124)"
                );
            }
        }
    }
    if let Some(value) = pipeline::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = pipelines::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = settings::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = project::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = skills::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = agents::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = mesh::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = git::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = agent::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = actor::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = surfaces::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = views::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = annotations::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    Err(CommandError::unknown_command(name))
}

/// Whether `name` is a registered write command (in the `COMMANDS` catalog).
/// The gate uses this to fail CLOSED: a known command with no resolvable blast
/// is denied, while a truly unknown name falls through to `unknown_command`.
fn is_known_command(name: &str) -> bool {
    crate::services::schema::COMMANDS.iter().any(|c| c.name == name)
}

/// The blast radius the gate enforces for `name`. Proto is the single source
/// (#67 a) — read by op name from the embedded descriptor; for commands not yet
/// on a per-domain proto service (`actor.*`, `mesh.*`, `project.*`,
/// `create.surface`, `annotation.post`, `feedback.post`) the hand-authored
/// `COMMANDS.blast_radius` is the declared authority (it is exactly what
/// `gen-ops` would project into proto). `None` only for a name that is neither
/// in proto nor in `COMMANDS` — a truly unknown op, which the gate ignores so it
/// can 404 downstream. The two sources never disagree where both carry a tier:
/// the `reflected_policy_matches_declared_commands` test pins that.
fn blast_for(name: &str) -> Option<crate::services::gate::BlastRadius> {
    use crate::services::gate::BlastRadius;
    if let Some(pol) = crate::services::op_policy::policy_for(name) {
        return Some(pol.blast);
    }
    crate::services::schema::COMMANDS
        .iter()
        .find(|c| c.name == name)
        .map(|c| BlastRadius::from_descriptor(c.blast_radius))
}

// ── arg helpers (shared by domain handlers) ──────────────────────────

/// The caller's agent id (identity-bound commands require it).
pub(crate) fn require_agent(ctx: &CommandCtx<'_>) -> Result<String, CommandError> {
    ctx.auth
        .agent_id
        .clone()
        .ok_or_else(|| CommandError::forbidden("agent identity required"))
}

pub(crate) fn req_str(args: &Value, key: &str) -> Result<String, CommandError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| CommandError::bad_args(format!("missing required arg `{key}`")))
}

pub(crate) fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// String-array arg (e.g. `paths`, `depends_on`).
pub(crate) fn str_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Bridge an interim axum-`Response` core into a command `Value`/error. The #78
/// service-lift removes this once those cores return typed values directly.
pub(crate) async fn response_to_value(
    resp: axum::response::Response,
) -> Result<Value, CommandError> {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .map_err(|e| CommandError::service(e.to_string()))?;
    if status.is_success() {
        Ok(serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    } else {
        Err(CommandError {
            status,
            code: if status.is_client_error() { "bad_args" } else { "service_error" },
            message: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_update_status_dispatches_through_the_registry() {
        let harness = crate::test_harness::TestDaemon::new().await;
        // The operator cred clears the gate (S6): mint it, don't rely on the
        // (now non-elevated) CLI transport.
        let auth = crate::test_harness::TestDaemon::operator_auth();
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({
            "pipeline_run_id": "run-cmd-test",
            "status": "gate_passed",
        });
        let value = call("pipeline.update_status", &args, &ctx)
            .await
            .expect("gate_passed has no required extra fields");
        assert_eq!(value["pipeline_run_id"], "run-cmd-test");
        assert_eq!(value["status"], "gate_passed");

        // Unknown command falls through to a structured error.
        let err = call("pipeline.nope", &args, &ctx).await.unwrap_err();
        assert_eq!(err.code, "unknown_command");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gate_denies_ungranted_agent_a_restricted_command() {
        let harness = crate::test_harness::TestDaemon::new().await;
        // A worker-role MCP agent: no orchestration grant, not the operator.
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("worker-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: ship_platform::auth::scopes::scopes_for_role(Some("worker")),
            grants: vec![],
            operator_cred: false,
        };
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({ "agent": "rust-lane" });
        // agent.dispatch is a restricted (capability-defining) op — denied
        // before any dispatch logic runs.
        let err = call("agent.dispatch", &args, &ctx).await.unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn every_known_command_resolves_to_a_blast_tier() {
        // FAIL CLOSED (#314): the gate denies a known command whose blast can't
        // be resolved. Prove the deny arm is unreachable for the real surface —
        // every registered command resolves to a tier via `blast_for` (proto
        // first, `COMMANDS` fallback), so no command runs ungated. A regression
        // that orphaned an op from BOTH sources would surface as a `None` here.
        for cmd in crate::services::schema::COMMANDS {
            assert!(is_known_command(cmd.name), "`{}` not seen as known", cmd.name);
            assert!(
                blast_for(cmd.name).is_some(),
                "known command `{}` has no resolvable blast — gate would deny (fail closed)",
                cmd.name
            );
        }
        // A name that is neither in proto nor in `COMMANDS` is unknown: no blast,
        // not force-denied — it falls through to `unknown_command` (404).
        assert!(!is_known_command("permissions.not_a_real_op"));
        assert!(blast_for("permissions.not_a_real_op").is_none());
    }

    #[test]
    fn every_c2_command_is_gated() {
        // SECURITY CONFORMANCE (#314): every command the catalog classifies
        // `C2_ELEVATED` resolves to C2 through `blast_for` — the value the gate
        // actually enforces — whether its policy comes from proto or the
        // `COMMANDS` fallback. None may resolve to a lower tier or to nothing
        // (which would be the fail-OPEN orphan #78 created). `agent.grant` (C3)
        // is correctly excluded — it's a higher tier, also gated.
        use crate::services::gate::BlastRadius;
        let mut bad = Vec::new();
        for cmd in crate::services::schema::COMMANDS {
            if BlastRadius::from_descriptor(cmd.blast_radius) != BlastRadius::C2Elevated {
                continue;
            }
            match blast_for(cmd.name) {
                Some(BlastRadius::C2Elevated) => {}
                other => bad.push(format!("{} → {other:?}", cmd.name)),
            }
        }
        assert!(bad.is_empty(), "C2 commands not gated as C2 (would fail OPEN): {bad:?}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gate_allows_agent_with_minted_orchestration_grant() {
        let harness = crate::test_harness::TestDaemon::new().await;
        // A commander-role agent: grants_for_role mints ship-orchestration into
        // the session, so the gate lets agent.dispatch past (it fails later on
        // missing args, NOT on the gate — proving the grant cleared it).
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("commander-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: ship_platform::auth::scopes::grants_for_role(Some("commander")),
            operator_cred: false,
        };
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({ "agent": "rust-lane" });
        // The grant cleared the gate: whether dispatch then succeeds or fails
        // downstream, it must NOT be a FORBIDDEN gate denial.
        if let Err(e) = call("agent.dispatch", &args, &ctx).await {
            assert_ne!(
                e.status,
                axum::http::StatusCode::FORBIDDEN,
                "minted orchestration grant should clear the gate"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agent_grant_upgrades_live_session_and_is_operator_only() {
        use ship_platform::auth::scopes::DaemonSession;
        let harness = crate::test_harness::TestDaemon::new().await;
        // A live worker session with no restricted grants.
        let token = "sess-grant-test";
        {
            let mut reg = harness.state.session_registry().lock().await;
            reg.insert(
                token.to_string(),
                DaemonSession {
                    token: token.to_string(),
                    agent_id: "worker-x".to_string(),
                    scopes: vec![],
                    grants: vec![],
                    workspace_id: None,
                    space_id: None,
                    pid: None,
                    created_at: std::time::Instant::now(),
                    expires_at_unix: None,
                    ephemeral: true,
                    is_operator_cred: false,
                },
            );
        }
        let args = serde_json::json!({ "agent": "worker-x", "grants": ["ship-orchestration"] });

        // Operator approves → the live session is upgraded. Operator authority
        // comes SOLELY from a verified operator cred (S6) — mint it.
        let op = crate::test_harness::TestDaemon::operator_auth();
        let v = call("agent.grant", &args, &CommandCtx { state: &harness.state, auth: &op })
            .await
            .expect("operator may grant");
        assert_eq!(v["sessions_upgraded"], 1);
        {
            let reg = harness.state.session_registry().lock().await;
            assert!(reg.get(token).unwrap().grants.iter().any(|g| g == "ship-orchestration"));
        }
        // #85: the approval also persists a provisioned baseline, so a future
        // re-dispatch (re-mint) reads it instead of falling back to the role.
        assert_eq!(
            ship_platform::services::permissions::projection::provisioned_grants_for("worker-x").unwrap(),
            Some(vec!["ship-orchestration".to_string()]),
            "operator approval must persist the provisioned baseline"
        );

        // A non-operator agent calling agent.grant is denied (C2 gate).
        let agent = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("x".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec!["ship-orchestration".to_string()],
            operator_cred: false,
        };
        let err = call("agent.grant", &args, &CommandCtx { state: &harness.state, auth: &agent })
            .await
            .unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);
    }

    #[test]
    fn artifact_record_value_decodes_into_the_typed_proto_message() {
        // The typed contract's load-bearing step: the generated Connect handler
        // maps the command's serde_json::Value onto the proto message via
        // from_value. ArtifactRecord exercises the risky shapes — a `uint64`
        // (size) as a JSON number, snake_case keys, and absent optionals — so
        // assert the round-trip the handler relies on actually holds.
        let value = serde_json::json!({
            "id": "art-1",
            "name": "out.log",
            "mime": "text/plain",
            "size": 4096,
            "path": "runs/r1/out.log",
            "scope": "run",
            "space_id": "proj",
            "run_id": "r1",
            "created_at": "2026-05-23T00:00:00Z"
            // content_hash / expires_at / owner_actor / collection_id / item_id
            // deliberately absent — proto3 optionals, must default to None.
        });
        let msg: ship_ops_rpc::ArtifactRecord =
            serde_json::from_value(value).expect("artifact value decodes into the proto message");
        assert_eq!(msg.id, "art-1");
        assert_eq!(msg.size, 4096);
        assert_eq!(msg.run_id.as_deref(), Some("r1"));
        assert_eq!(msg.content_hash, None);
    }

    #[test]
    fn collection_item_value_decodes_into_the_typed_proto_message() {
        // The open `data` payload rides as a `data_json` string; the envelope is
        // typed. Assert the handler's from_value round-trip holds, including an
        // absent optional (created_by).
        let value = serde_json::json!({
            "id": "item-1",
            "collection": "notes",
            "data_json": "{\"title\":\"hi\"}",
            "created_at": "2026-05-23T00:00:00Z",
            "updated_at": "2026-05-23T00:00:00Z"
        });
        let msg: ship_ops_rpc::CollectionItem =
            serde_json::from_value(value).expect("collection item value decodes");
        assert_eq!(msg.id, "item-1");
        assert_eq!(msg.collection, "notes");
        assert_eq!(msg.data_json, "{\"title\":\"hi\"}");
        assert_eq!(msg.created_by, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn skills_create_is_restricted_to_ship_authoring() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let args = serde_json::json!({ "id": "fixture-skill", "scope": "user" });

        // A worker (no grants) can't author a skill — skills are capability
        // groups, gated behind the ship-authoring grant.
        let worker = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("worker-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
            operator_cred: false,
        };
        let err = call("skills.create", &args, &CommandCtx { state: &harness.state, auth: &worker })
            .await
            .unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);

        // A designer holding ship-authoring clears the gate and writes through.
        let designer = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("designer-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec!["skills".to_string()],
            operator_cred: false,
        };
        let created =
            call("skills.create", &args, &CommandCtx { state: &harness.state, auth: &designer })
                .await
                .expect("the `skills` capability clears the gate and the skill is created");
        assert_eq!(created["id"], "fixture-skill");
        assert_eq!(created["scope"], "user");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agents_create_needs_authoring_not_orchestration_grant() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let args = serde_json::json!({
            "id": "fixture-agent",
            "doc_json": "{\"role\":\"worker\"}",
            "scope": "user"
        });

        // A commander holds ship-orchestration (dispatch/steer) — but authoring a
        // profile is ship-authoring. The singular/plural prefix split means the
        // orchestration grant does NOT clear agents.create.
        let commander = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("commander-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec!["ship-orchestration".to_string()],
            operator_cred: false,
        };
        let err =
            call("agents.create", &args, &CommandCtx { state: &harness.state, auth: &commander })
                .await
                .unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);

        // A designer holds ship-authoring → cleared, writes through.
        let designer = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("designer-1".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec!["agents".to_string()],
            operator_cred: false,
        };
        let created =
            call("agents.create", &args, &CommandCtx { state: &harness.state, auth: &designer })
                .await
                .expect("the `agents` capability clears agents.create");
        assert_eq!(created["id"], "fixture-agent");
        assert_eq!(created["scope"], "user");
    }

    #[test]
    fn typed_read_values_decode_into_their_proto_messages() {
        // The read half of the typed contract: dispatch_typed decodes the
        // registry's JSON projection onto the proto message. Exercise the new
        // shapes — a nested repeated message (RulesIndexResponse → IndexEntry)
        // and a flat record (RuleGetResponse) — the way the generated read
        // handler does.
        let index = serde_json::json!({
            "items": [ { "id": "no-slop", "scope": "user" }, { "id": "style", "scope": "project" } ]
        });
        let msg: ship_ops_rpc::RulesIndexResponse =
            serde_json::from_value(index).expect("rules.list value decodes");
        assert_eq!(msg.items.len(), 2);
        assert_eq!(msg.items[0].id, "no-slop");
        assert_eq!(msg.items[1].scope, "project");

        let detail = serde_json::json!({ "id": "no-slop", "scope": "user", "body": "# No slop" });
        let got: ship_ops_rpc::RuleGetResponse =
            serde_json::from_value(detail).expect("rules.get value decodes");
        assert_eq!(got.id, "no-slop");
        assert_eq!(got.body, "# No slop");
    }
}
