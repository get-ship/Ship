//! Tools = thin routers over the two rails, each passing through the spine.
//! Every `call()` is `spine::run(tier, auth, execute, verify)` — so the permit
//! and verify gates are impossible to bypass.

use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::json;

use crate::auth::AuthContext;
use crate::context::{ContextStore, ScriptEngine};
use crate::gate::Gate;
use crate::rails::Sandbox;
use crate::spine::{self, Blast, SpineError};
use crate::transport::OpsTransport;

// --------------------------------------------------------------------------- read_file (C0)
// Context-mode: large reads are OFFLOADED to the ContextStore — the model gets a
// compact handle + preview, not the whole file, and pulls slices on demand.
pub struct ReadFile {
    pub gate: Arc<Gate>,
    pub store: Arc<ContextStore>,
}
#[derive(Deserialize)]
pub struct ReadArgs {
    path: String,
}
impl Tool for ReadFile {
    const NAME: &'static str = "read_file";
    type Error = SpineError;
    type Args = ReadArgs;
    type Output = String;

    async fn definition(&self, _p: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Read a UTF-8 text file. Large files return a context handle + preview; \
                          retrieve slices instead of loading everything."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, a: ReadArgs) -> Result<String, SpineError> {
        let store = self.store.clone();
        spine::run(
            &self.gate,
            "read_file",
            Blast::C0Safe,
            || std::fs::read_to_string(&a.path).map_err(|e| e.to_string()),
            move |contents| {
                if contents.len() > store.threshold() {
                    let h = store.put(contents.clone());
                    Ok(format!(
                        "[offloaded {} bytes -> {} | preview:\n{}\n... retrieve slices via context_read({})]",
                        h.bytes, h.id, h.preview, h.id
                    ))
                } else {
                    Ok(contents.clone())
                }
            },
        )
    }
}

// --------------------------------------------------------------------------- edit_file (C1)
// The showcase for the verify gate: after writing, we RE-READ and confirm the
// edit actually landed. If not, the call is refused — not reported as success.
pub struct EditFile {
    pub gate: Arc<Gate>,
}
#[derive(Deserialize)]
pub struct EditArgs {
    path: String,
    old: String,
    new: String,
}
impl Tool for EditFile {
    const NAME: &'static str = "edit_file";
    type Error = SpineError;
    type Args = EditArgs;
    type Output = String;

    async fn definition(&self, _p: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Replace the first occurrence of `old` with `new` in a file.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "old":  { "type": "string" },
                    "new":  { "type": "string" }
                },
                "required": ["path", "old", "new"]
            }),
        }
    }

    async fn call(&self, a: EditArgs) -> Result<String, SpineError> {
        let path = a.path.clone();
        let new = a.new.clone();
        spine::run(
            &self.gate,
            "edit_file",
            Blast::C1Space,
            // EXECUTE
            move || {
                let src = std::fs::read_to_string(&a.path).map_err(|e| e.to_string())?;
                if !src.contains(&a.old) {
                    return Err(format!("`old` not found in {}", a.path));
                }
                let edited = src.replacen(&a.old, &a.new, 1);
                std::fs::write(&a.path, &edited).map_err(|e| e.to_string())?;
                Ok(())
            },
            // VERIFY — re-read from disk, don't trust the write call's return.
            move |_| {
                let on_disk = std::fs::read_to_string(&path)
                    .map_err(|e| format!("post-edit re-read failed: {e}"))?;
                if on_disk.contains(&new) {
                    Ok(format!("edit verified on disk in {path}"))
                } else {
                    Err(format!("edit did not land in {path}; context not updated"))
                }
            },
        )
    }
}

// --------------------------------------------------------------------------- bash (C1, Rail B)
// Routes to the Sandbox. The default sandbox refuses host exec, so this tool is
// inert until a Ship isolation tier is attached. Verify surfaces the REAL exit
// code: a nonzero exit is refused, never quietly accepted.
pub struct Bash {
    pub gate: Arc<Gate>,
    pub sandbox: Arc<dyn Sandbox>,
}
#[derive(Deserialize)]
pub struct BashArgs {
    cmd: String,
}
impl Tool for Bash {
    const NAME: &'static str = "bash";
    type Error = SpineError;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _p: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Run a shell command inside the isolation sandbox.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "cmd": { "type": "string" } },
                "required": ["cmd"]
            }),
        }
    }

    async fn call(&self, a: BashArgs) -> Result<String, SpineError> {
        let sandbox = self.sandbox.clone();
        spine::run(
            &self.gate,
            "bash",
            Blast::C1Space,
            move || sandbox.run(&a.cmd),
            |out| {
                if out.code == 0 {
                    Ok(out.stdout.clone())
                } else {
                    Err(format!(
                        "command exited {}: {}",
                        out.code,
                        out.stderr.trim()
                    ))
                }
            },
        )
    }
}

// --------------------------------------------------------------------------- ship_op (Rail A)
// The structured rail: the harness does not implement git/artifacts, it
// dispatches over the transport, which carries the constructed AuthContext. The
// gate (capability + blast radius) lives in shipd at `command::call`, reached
// through the transport's Gateway seam — NOT re-implemented here, and untouched
// while the ops SSOT is mid-refactor.
pub struct ShipOp {
    pub identity: AuthContext,
    pub transport: Arc<dyn OpsTransport>,
}
#[derive(Deserialize)]
pub struct OpArgs {
    op: String,
    #[serde(default)]
    args: serde_json::Value,
}
impl Tool for ShipOp {
    const NAME: &'static str = "ship_op";
    type Error = SpineError;
    type Args = OpArgs;
    type Output = String;

    async fn definition(&self, _p: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Call a structured Ship op (e.g. git.status, collections.put). \
                          Use this instead of running git or editing platform state directly."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "op":   { "type": "string", "description": "dotted op name, e.g. git.status" },
                    "args": { "type": "object" }
                },
                "required": ["op"]
            }),
        }
    }

    async fn call(&self, a: OpArgs) -> Result<String, SpineError> {
        // No local spine gate here: ops are gated inside shipd at command::call.
        // The transport carries our identity; the gate decides.
        match self.transport.call(&a.op, a.args, &self.identity).await {
            Ok(value) => Ok(value.to_string()),
            Err(e) => Err(SpineError::Execute(e.to_string())),
        }
    }
}

// --------------------------------------------------------------------------- run_script (execution)
// The context-mode execution primitive for a no-bash agent: the model writes JS,
// it runs in the rquickjs sandbox (in-process, no host), and ONLY stdout enters
// context. Same engine hosts the Ship JS SDK for agent orchestration.
pub struct RunScript {
    pub gate: Arc<Gate>,
    pub engine: Arc<dyn ScriptEngine>,
}
#[derive(Deserialize)]
pub struct ScriptArgs {
    src: String,
}
impl Tool for RunScript {
    const NAME: &'static str = "run_script";
    type Error = SpineError;
    type Args = ScriptArgs;
    type Output = String;

    async fn definition(&self, _p: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Run a JS snippet in a sandboxed VM and return only its stdout. \
                          Prefer this over reading lots of data: compute the answer, log it."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": { "src": { "type": "string", "description": "JS source; log results" } },
                "required": ["src"]
            }),
        }
    }

    async fn call(&self, a: ScriptArgs) -> Result<String, SpineError> {
        let engine = self.engine.clone();
        spine::run(
            &self.gate,
            "run_script",
            Blast::C1Space,
            move || engine.run(&a.src).map_err(|e| e.to_string()),
            |stdout| Ok(stdout.clone()),
        )
    }
}
