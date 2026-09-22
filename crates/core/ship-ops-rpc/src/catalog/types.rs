//! Descriptor types shared by the op catalog and command catalog.

use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct OpDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub returns: &'static str,
    pub permission: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

#[derive(Serialize, Clone, Copy)]
pub struct ArgDescriptor {
    pub name: &'static str,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub required: bool,
}

pub(super) fn is_false(v: &bool) -> bool {
    !v
}

/// A mutation op. The write-side analog of `OpDescriptor`: same shape minus
/// `returns` (commands return a status JSON), plus `blast_radius` for the gate
/// + the generated proto policy. One source for the write surface, exactly as
/// `OPS` is for reads.
#[derive(Serialize, Clone, Copy)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub permission: &'static str,
    /// Gate class: "C1_WORKSPACE" | "C2_ELEVATED".
    pub blast_radius: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
    /// Typed proto response message. Empty = `JsonResponse` (the legacy blob,
    /// for not-yet-typed ops). When set, `gen-ops` emits `returns ({response_type})`
    /// and the impl maps the command's `Value` → the typed message via
    /// `serde_json::from_value`. See `runtime-op-surface-declaration.md`.
    #[serde(skip_serializing_if = "str::is_empty")]
    pub response_type: &'static str,
}
