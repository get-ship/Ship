//! The op ABI — one model for reads and commands.
//!
//! An [`Op`] is a descriptor (name, args, return types, blast class) plus a
//! [`Handler`]. The read/command distinction is the handler *variant*, not two
//! parallel subsystems: a read is a pure synchronous projection query; a command
//! is an effectful async mutation. The gate class ([`BlastRadius`]) is a field,
//! and a read pins to [`BlastRadius::C0Read`] by construction.
//!
//! This crate is the shared vocabulary depended on by every crate that *declares*
//! ops (platform services today, extensions later). It holds **no business logic
//! and no transport** — handlers receive a host-provided context `C` and return
//! `serde_json::Value`. The composition root (shipd) builds a [`Registry`],
//! enforces the gate against each [`Op::blast`], and serves the wire.
//!
//! Generic over `C` so the context (which aggregates the platform services a
//! handler needs) is injected by the layer that owns those services — this crate
//! never names them, keeping it a true leaf.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::Serialize;
use serde_json::Value;

// ── Errors ────────────────────────────────────────────────────────────────

/// Unifies the old `RegistryError` (reads) + `CommandError` (commands): same
/// variants, one type. Host transports map these onto their wire error codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    /// The named entity does not exist.
    NotFound(String),
    /// The arguments were malformed or failed validation.
    BadArgs(String),
    /// The handler's underlying service failed.
    Service(String),
    /// The gate denied the call (caller lacks the capability).
    Denied(String),
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::NotFound(m) => write!(f, "not found: {m}"),
            OpError::BadArgs(m) => write!(f, "bad arguments: {m}"),
            OpError::Service(m) => write!(f, "service error: {m}"),
            OpError::Denied(m) => write!(f, "denied: {m}"),
        }
    }
}

impl std::error::Error for OpError {}

// ── Descriptor metadata ─────────────────────────────────────────────────────

/// One declared argument. Mirrors the prior `ArgDescriptor` verbatim.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ArgDescriptor {
    pub name: &'static str,
    /// Wire kind, e.g. `"string"`, `"string?"`, `"u64"`.
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub required: bool,
}

fn is_false(v: &bool) -> bool {
    !v
}

/// Blast-radius class — the human-vs-agent line the gate enforces. Carried on
/// the descriptor (declared by the op's owner); the gate (in the composition
/// root) reads it and decides. A read is `C0Read` by construction.
///
/// The wire/string form (`"C0_READ"` etc.) is the canonical descriptor encoding,
/// kept for compatibility with the existing schema endpoint and codegen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BlastRadius {
    /// Read / own-scope. Harmless to Ship.
    #[serde(rename = "C0_READ")]
    C0Read,
    /// Mutates the caller's own workspace — the user's own risk.
    #[serde(rename = "C1_WORKSPACE")]
    C1Workspace,
    /// Ship/cloud-dangerous (credential mint, billing). Human-operator only.
    #[serde(rename = "C2_ELEVATED")]
    C2Elevated,
}

impl BlastRadius {
    /// Parse the descriptor's string form. An unrecognized value is treated as
    /// `C2Elevated` — **fail closed**: a misdeclared op is gated to the human,
    /// never silently opened to agents.
    pub fn from_descriptor(s: &str) -> Self {
        match s {
            "C0_READ" => Self::C0Read,
            "C1_WORKSPACE" => Self::C1Workspace,
            "C2_ELEVATED" => Self::C2Elevated,
            _ => Self::C2Elevated,
        }
    }

    /// The canonical string form used on the wire and in codegen.
    pub fn as_descriptor(self) -> &'static str {
        match self {
            Self::C0Read => "C0_READ",
            Self::C1Workspace => "C1_WORKSPACE",
            Self::C2Elevated => "C2_ELEVATED",
        }
    }
}

/// Serializable projection of an [`Op`] — everything but the handler. This is
/// what the schema endpoint and SDK codegen consume; the handler (a fn pointer)
/// is never serialized.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct OpMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    /// The handler's Rust return type (via `stringify!`) — drives `DataHandle<T>`.
    pub returns: &'static str,
    /// Proto message name; `""` = `JsonResponse` (not-yet-typed blob).
    #[serde(skip_serializing_if = "str::is_empty")]
    pub response_type: &'static str,
    pub blast: BlastRadius,
    /// `true` for reads, `false` for commands — derived from the handler variant.
    pub read: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

// ── Handler + Op ────────────────────────────────────────────────────────────

/// A pure synchronous read handler — a projection query. C0 by construction.
pub type ReadFn<C> = fn(&Value, &C) -> Result<Value, OpError>;

/// The boxed future a command handler returns, borrowing args + ctx for `'a`.
pub type CommandFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, OpError>> + Send + 'a>>;

/// An effectful asynchronous command handler. Today it mutates a service; at the
/// substrate inversion it appends events + an applier materializes the effect.
pub type CommandFn<C> = for<'a> fn(&'a Value, &'a C) -> CommandFuture<'a>;

/// The read/command distinction, as a type. There is no `kind` field — which
/// variant you supply *is* the distinction.
pub enum Handler<C> {
    Read(ReadFn<C>),
    Command(CommandFn<C>),
}

/// One op: descriptor + handler. Const-constructible so domains declare static
/// `&[Op<C>]` tables.
pub struct Op<C> {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub returns: &'static str,
    pub response_type: &'static str,
    pub blast: BlastRadius,
    pub sensitive: bool,
    pub handler: Handler<C>,
}

impl<C> Op<C> {
    /// `true` if this op is a pure read.
    pub fn is_read(&self) -> bool {
        matches!(self.handler, Handler::Read(_))
    }

    /// The serializable metadata projection (no handler).
    pub fn meta(&self) -> OpMeta {
        OpMeta {
            name: self.name,
            description: self.description,
            args: self.args,
            returns: self.returns,
            response_type: self.response_type,
            blast: self.blast,
            read: self.is_read(),
            sensitive: self.sensitive,
        }
    }

    /// Run the handler. The composition root enforces the gate against
    /// [`Op::blast`] *before* calling this — this crate does not gate.
    pub async fn run(&self, args: &Value, ctx: &C) -> Result<Value, OpError> {
        match self.handler {
            Handler::Read(f) => f(args, ctx),
            Handler::Command(f) => f(args, ctx).await,
        }
    }
}

// ── Registry ──────────────────────────────────────────────────────────────

/// A name→op map assembled once from the domains' static tables. Replaces the
/// linear dispatch chains with O(1) lookup. The composition root mounts each
/// declaring crate's `&[Op<C>]` into one registry.
pub struct Registry<C: 'static> {
    by_name: HashMap<&'static str, &'static Op<C>>,
}

impl<C: 'static> Default for Registry<C> {
    fn default() -> Self {
        Self { by_name: HashMap::new() }
    }
}

impl<C: 'static> Registry<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mount a domain's static op table. Returns `Err(name)` on a duplicate op
    /// name — a declaration collision is a build-time bug, surfaced loudly.
    pub fn mount(&mut self, ops: &'static [Op<C>]) -> Result<(), &'static str> {
        for op in ops {
            if self.by_name.insert(op.name, op).is_some() {
                return Err(op.name);
            }
        }
        Ok(())
    }

    /// Look up an op by name.
    pub fn get(&self, name: &str) -> Option<&'static Op<C>> {
        self.by_name.get(name).copied()
    }

    /// Every mounted op's metadata — the source for the schema endpoint + codegen.
    pub fn metas(&self) -> Vec<OpMeta> {
        let mut v: Vec<OpMeta> = self.by_name.values().map(|op| op.meta()).collect();
        v.sort_by_key(|m| m.name);
        v
    }

    /// Number of mounted ops.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

#[cfg(test)]
mod tests;
