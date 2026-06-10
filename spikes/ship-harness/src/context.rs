//! Context economy — the "context-mode" idea, adapted to a no-bash agent.
//!
//! Two bones:
//!
//! 1. ContextStore — large tool outputs do NOT go into the model's context. They
//!    are stored; the tool returns a compact Handle (id + preview + size) and the
//!    model pulls only the slices it needs via search/read. This is the 98%
//!    reduction: "treat the model as a code generator / retriever, not a data
//!    processor." Buildable now in-memory; real impl is SQLite FTS5 + BM25 + TTL.
//!
//! 2. ScriptEngine — the execution substrate. With no sidecar/bash, the agent's
//!    "run code" primitive is **rquickjs**: an in-process, sandboxed JS VM with no
//!    host access. The model writes JS, it runs in the VM, ONLY stdout enters
//!    context. Memory-safe code execution on the host without virtualization.
//!
//!    The same engine is the ORCHESTRATION host: a Ship JS SDK runs here where
//!    `agent()`, `parallel()`, `pipeline()` are bound to the AgentRuntime — so a
//!    pipeline step can be a JS workflow that fans out agents, Claude-Workflow
//!    style. One VM, two jobs: tool-scripts and agent orchestration.

use std::collections::HashMap;
use std::sync::Mutex;

// --------------------------------------------------------------------------- ContextStore

/// A compact reference to offloaded content. This is what enters context.
#[derive(Debug, Clone)]
pub struct Handle {
    pub id: String,
    pub bytes: usize,
    pub preview: String,
}

/// Offload-and-retrieve store. Anything over `threshold` bytes is stored, not
/// inlined.
pub struct ContextStore {
    threshold: usize,
    blobs: Mutex<HashMap<String, String>>,
    seq: Mutex<u64>,
}

impl ContextStore {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            blobs: Mutex::new(HashMap::new()),
            seq: Mutex::new(0),
        }
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Store content, return a Handle (id + head preview + size). The model sees
    /// the Handle, never the full content, until it retrieves slices.
    pub fn put(&self, content: String) -> Handle {
        let id = {
            let mut s = self.seq.lock().unwrap();
            *s += 1;
            format!("ctx_{:04}", *s)
        };
        let preview: String = content.lines().take(3).collect::<Vec<_>>().join("\n");
        let preview: String = preview.chars().take(240).collect();
        let bytes = content.len();
        self.blobs.lock().unwrap().insert(id.clone(), content);
        Handle { id, bytes, preview }
    }

    /// Retrieve the lines of `id` matching `query` (naive; real = BM25). Returns
    /// only the hits — the point is to pull slices, not the whole blob.
    pub fn search(&self, id: &str, query: &str) -> Vec<String> {
        let q = query.to_lowercase();
        self.blobs
            .lock()
            .unwrap()
            .get(id)
            .map(|c| {
                c.lines()
                    .filter(|l| l.to_lowercase().contains(&q))
                    .map(|l| l.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Read a line range [start, start+len) of `id`.
    pub fn read(&self, id: &str, start: usize, len: usize) -> Option<String> {
        self.blobs.lock().unwrap().get(id).map(|c| {
            c.lines()
                .skip(start)
                .take(len)
                .collect::<Vec<_>>()
                .join("\n")
        })
    }
}

// --------------------------------------------------------------------------- ScriptEngine

#[derive(Debug)]
pub enum ScriptError {
    Unavailable(String),
    Runtime(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Unavailable(m) => write!(f, "script engine unavailable: {m}"),
            ScriptError::Runtime(m) => write!(f, "script error: {m}"),
        }
    }
}
impl std::error::Error for ScriptError {}

/// Run a script in a sandboxed VM; return ONLY stdout. The real impl is rquickjs
/// (in-process, no host, no network). It is also the orchestration host for the
/// Ship JS SDK.
pub trait ScriptEngine: Send + Sync {
    fn run(&self, src: &str) -> Result<String, ScriptError>;
    fn label(&self) -> &'static str;
}

/// Until rquickjs is wired, scripts are unavailable — but the seam (and the
/// `run_script` tool) is in place. NB: unlike bash, this needs no sidecar; it's
/// a host-safe in-process VM, so wiring it does not gate on the coding-agent
/// isolation pass.
pub struct PendingScriptEngine;
impl ScriptEngine for PendingScriptEngine {
    fn run(&self, _src: &str) -> Result<String, ScriptError> {
        Err(ScriptError::Unavailable(
            "rquickjs not yet wired — host-safe, no sidecar required".into(),
        ))
    }
    fn label(&self) -> &'static str {
        "pending-rquickjs"
    }
}
