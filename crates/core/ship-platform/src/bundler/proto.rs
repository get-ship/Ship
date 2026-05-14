//! JSON-line protocol types matching bundler.ts.
//!
//! The protocol is the contract — the Rust side never holds shared in-memory
//! types with the worker beyond this JSON shape. Alternative workers slot in
//! without Rust changes.

use serde::{Deserialize, Serialize};

// ── Requests (shipd → worker) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerRequest {
    Build { id: String, target: String },
    BuildAll { id: String },
    WatchStart { id: String },
    WatchStop { id: String },
    Shutdown { id: String },
}

impl WorkerRequest {
    pub fn id(&self) -> &str {
        match self {
            Self::Build { id, .. } => id,
            Self::BuildAll { id } => id,
            Self::WatchStart { id } => id,
            Self::WatchStop { id } => id,
            Self::Shutdown { id } => id,
        }
    }
}

// ── Responses (worker → shipd, correlated by id) ─────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerResponse {
    Bundle {
        id: String,
        target: String,
        bytes: usize,
        #[serde(rename = "outPath")]
        out_path: String,
        manifest: BundleManifest,
    },
    Ack {
        id: String,
    },
    Error {
        id: String,
        target: Option<String>,
        message: String,
        stack: Option<String>,
    },
}

impl WorkerResponse {
    pub fn id(&self) -> &str {
        match self {
            Self::Bundle { id, .. } => id,
            Self::Ack { id } => id,
            Self::Error { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BundleManifest {
    pub target: String,
    #[serde(rename = "outPath")]
    pub out_path: String,
    pub bytes: usize,
    pub inputs: Vec<String>,
}

// ── Events (worker → shipd, unsolicited while watching) ──────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerEvent {
    ModuleChanged {
        target: String,
        #[serde(rename = "outPath")]
        out_path: String,
        bytes: usize,
    },
    CompileError {
        target: String,
        message: String,
        line: Option<u32>,
        col: Option<u32>,
        file: Option<String>,
    },
    Ready,
}

// ── Incoming line (either a response or an unsolicited event) ────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WorkerLine {
    Response(WorkerResponse),
    Event(WorkerEvent),
}
