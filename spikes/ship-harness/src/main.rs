//! ship-harness-spike — Phase-1 proof of concept for a first-party Rust coding
//! agent for Ship.
//!
//! Proves three things from the design brief:
//!   1. SUBSTRATE: rig drives the Anthropic + OpenAI-compatible agent loop.
//!   2. SPINE:     every tool call passes Permit -> Execute -> Verify (spine.rs).
//!   3. RAILS:     structured work -> ship ops; shell -> sandbox; host bash is
//!                 refused by construction (rails.rs).
//!
//! Run:
//!   SHIP_PROVIDER=anthropic ANTHROPIC_API_KEY=... cargo run
//!   SHIP_PROVIDER=openai-compat OPENAI_API_KEY=... SHIP_BASE_URL=http://localhost:11434/v1 \
//!     SHIP_MODEL=llama3.1 cargo run
//!   # opt into host bash for local exercise only:
//!   SHIP_SPIKE_ALLOW_HOST_BASH=1 ...

// Architecture spike: several modules define seams (CompactionPolicy,
// WorkingMemory, RunMode, OutputContract::Json, DenyOnAsk, ContextStore::read)
// that are intentionally defined ahead of being wired. Allow the dead-code these
// bones produce until the corresponding pass lands.
#![allow(dead_code)]

mod auth;
mod context;
mod gate;
mod memory;
mod provider;
mod rails;
mod runtime;
mod session;
mod sink;
mod spec;
mod spine;
mod tools;
mod transport;

use std::io::{self, Write};
use std::sync::Arc;

use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::CompletionModel;
use rig::message::Message;
use rig::providers::{anthropic, openai};
use rig::tool::Tool;
use serde_json::json;

use auth::AuthContext;
use context::{ContextStore, PendingScriptEngine, ScriptEngine};
use gate::{AutoApprove, Gate};
use memory::{InMemoryMemory, Memory};
use provider::{Credential, CredentialSource, Provider};
use rails::{sandbox_from_env, Sandbox};
use session::run_turn;
use sink::TerminalSink;
use spec::AgentSpec;
use spine::Authority;
use tools::{Bash, EditFile, ReadFile, RunScript, ShipOp};
use transport::{InProcessTransport, OpsTransport, StubGateway};

const PREAMBLE: &str = "\
You are Ship's first-party coding agent. You have four tools:
- read_file: read workspace files.
- edit_file: change a file (the harness re-reads to verify the edit landed).
- bash: run a shell command INSIDE the isolation sandbox. Never assume host access.
- ship_op: call structured Ship ops (git.status, git.commit, artifacts.patch, ...).
  Always use ship_op for git and platform actions — never shell out to git.
If a tool returns DENIED or UNVERIFIED, do not pretend it succeeded; adjust and retry.";

/// Attach the spine-gated tool set to a provider's agent builder and build it.
macro_rules! build_agent {
    ($builder:expr, $gate:expr, $sandbox:expr, $store:expr, $engine:expr, $identity:expr, $transport:expr) => {
        $builder
            .preamble(PREAMBLE)
            .tool(ReadFile {
                gate: $gate.clone(),
                store: $store,
            })
            .tool(EditFile { gate: $gate.clone() })
            .tool(Bash {
                gate: $gate.clone(),
                sandbox: $sandbox,
            })
            .tool(RunScript {
                gate: $gate.clone(),
                engine: $engine,
            })
            .tool(ShipOp {
                identity: $identity,
                transport: $transport,
            })
            .build()
    };
}

/// Interactive REPL that drives the agent through the streamed `AgentEvent`
/// channel (session.rs) into a sink. This is the user-facing interface: in the
/// product the same events feed a SUI view; here a TerminalSink renders them.
async fn repl<M>(
    agent: Agent<M>,
    first: Option<String>,
    memory: &dyn Memory,
) -> anyhow::Result<()>
where
    M: CompletionModel + 'static,
{
    let session = "interactive";
    let mut history: Vec<Message> = memory.load(session);
    let sink = TerminalSink;
    let stdin = io::stdin();
    let mut pending = first;

    loop {
        let line = match pending.take() {
            Some(p) => p,
            None => {
                print!("\nyou> ");
                io::stdout().flush().ok();
                let mut buf = String::new();
                if stdin.read_line(&mut buf)? == 0 {
                    break; // EOF (Ctrl-D)
                }
                buf.trim().to_string()
            }
        };
        if line.is_empty() {
            continue;
        }
        if line == "/exit" || line == "/quit" {
            break;
        }

        print!("\nagent> ");
        io::stdout().flush().ok();
        run_turn(&agent, &line, &mut history, &sink).await;
        memory.save(session, &history); // persist after each turn
    }
    Ok(())
}

/// Deterministic, no-API drive of the spine + rails. Invokes the tools directly
/// so every gate decision is visible. Run: `SHIP_PROVIDER=selftest cargo run`.
#[allow(clippy::too_many_arguments)]
async fn selftest(
    gate: Arc<Gate>,
    sandbox: Arc<dyn Sandbox>,
    store: Arc<ContextStore>,
    engine: Arc<dyn ScriptEngine>,
    identity: AuthContext,
    transport: Arc<dyn OpsTransport>,
) -> anyhow::Result<()> {
    let report = |label: &str, r: Result<String, spine::SpineError>| match r {
        Ok(ok) => println!("[{label}] ACCEPTED: {}\n", ok.lines().next().unwrap_or("").trim()),
        Err(e) => println!("[{label}] {e}\n"),
    };

    // 1. read_file (C0) — small file inlines; large file OFFLOADS to ContextStore.
    let read = ReadFile { gate: gate.clone(), store: store.clone() };
    let big = std::env::temp_dir().join("ship_spike_big.txt");
    std::fs::write(&big, "line\n".repeat(500))?; // > 1KB threshold
    report(
        "read large file (context offload)",
        read.call(serde_json::from_value(json!({ "path": big.to_string_lossy() }))?).await,
    );

    // 2. edit_file (C1) — in base policy this is `ask`; AutoApprove grants it,
    // then the verify gate confirms the edit landed on disk.
    let tmp = std::env::temp_dir().join("ship_spike_demo.txt");
    std::fs::write(&tmp, "hello world\n")?;
    let edit = EditFile { gate: gate.clone() };
    report(
        "edit_file (ask -> approved -> verified)",
        edit.call(serde_json::from_value(json!({
            "path": tmp.to_string_lossy(), "old": "world", "new": "ship"
        }))?)
        .await,
    );

    // 3. bash — DENIED by agent policy (base spec denies bash; no sidecar). The
    // permit gate stops it before the sandbox is even consulted.
    let bash = Bash { gate: gate.clone(), sandbox };
    report(
        "bash (denied by policy)",
        bash.call(serde_json::from_value(json!({ "cmd": "echo hi" }))?).await,
    );

    // 4. run_script — the rquickjs execution seam. Unavailable until wired, but
    // host-safe (no sidecar needed). This is context-mode's "model as coder".
    let script = RunScript { gate: gate.clone(), engine };
    report(
        "run_script (rquickjs pending)",
        script.call(serde_json::from_value(json!({ "src": "console.log(2+2)" }))?).await,
    );

    // 5. ship_op (Rail A) — routes over the in-process transport with identity.
    let op = ShipOp { identity, transport };
    report(
        "ship_op git.status (in-process transport)",
        op.call(serde_json::from_value(json!({ "op": "git.status" }))?).await,
    );

    // 6. context retrieval — pull a slice from the offloaded blob, not the whole.
    let hits = store.search("ctx_0001", "line");
    println!("[context_search ctx_0001 'line'] {} matching lines (slice, not the blob)\n", hits.len());

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = std::env::var("SHIP_PROVIDER").unwrap_or_else(|_| "anthropic".into());
    let opening = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    let opening = if opening.is_empty() { None } else { Some(opening) };

    // The programmable spec — in prod projected from .ship/agents/<id>.jsonc.
    let spec = AgentSpec::base("ship-agent-demo", "");

    // Local-effect ceiling for file/script tools. Ops are gated separately in
    // shipd (by AuthContext), not by this ceiling.
    let auth = Authority {
        ceiling: spec.blast_ceiling,
    };

    // Tool gate = spec policy + ceiling + approver. AutoApprove here resolves the
    // `ask` stance (edit_file); a headless pipeline step uses DenyOnAsk.
    let gate = Arc::new(Gate::new(
        auth,
        spec.tool_policy.clone(),
        Arc::new(AutoApprove),
    ));

    // Context economy + execution + memory bones.
    let store = Arc::new(ContextStore::new(1024)); // offload >1KB
    let engine: Arc<dyn ScriptEngine> = Arc::new(PendingScriptEngine);
    let memory = InMemoryMemory::default();

    // The THIRD auth provenance: identity constructed in-process at dispatch.
    let identity = AuthContext::in_process(&spec.id, vec!["git".into()]);

    let sandbox: Arc<dyn Sandbox> = Arc::from(sandbox_from_env());
    let transport: Arc<dyn OpsTransport> = Arc::new(InProcessTransport {
        gateway: Arc::new(StubGateway),
    });

    eprintln!("agent        = {}", spec.id);
    eprintln!("provider     = {provider}");
    eprintln!("sandbox rail = {}", sandbox.label());
    eprintln!("script engine= {}", engine.label());
    eprintln!("ops transport= {}", transport.kind());
    eprintln!(
        "identity     = {:?}  grants={:?}  operator={}",
        identity.agent_id, identity.grants, identity.operator_cred
    );
    eprintln!(
        "tool policy  = deny{:?} ask{:?}  ceiling={:?}   (/exit, Ctrl-D)\n",
        spec.tool_policy.deny, spec.tool_policy.ask, auth.ceiling
    );

    if provider == "selftest" {
        selftest(gate, sandbox, store, engine, identity, transport).await?;
        return Ok(());
    }

    // BYOK at launch — resolved behind the CredentialSource seam. A future
    // ShipHostedCredentials impl drops in here unchanged.
    let cred: Credential = provider::Byok.resolve()?;
    eprintln!("model        = {}\n", cred.model);

    match cred.provider {
        Provider::Anthropic => {
            let client = anthropic::Client::builder().api_key(&cred.api_key).build()?;
            let agent = build_agent!(
                client.agent(&cred.model),
                gate,
                sandbox.clone(),
                store.clone(),
                engine.clone(),
                identity.clone(),
                transport.clone()
            );
            repl(agent, opening, &memory).await?;
        }
        Provider::OpenAiCompat { base_url } => {
            // Completions API client = the de-facto OpenAI-compatible surface.
            let client = openai::CompletionsClient::builder()
                .api_key(&cred.api_key)
                .base_url(&base_url)
                .build()?;
            let agent = build_agent!(
                client.agent(&cred.model),
                gate,
                sandbox.clone(),
                store.clone(),
                engine.clone(),
                identity.clone(),
                transport.clone()
            );
            repl(agent, opening, &memory).await?;
        }
    }
    Ok(())
}
