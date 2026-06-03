//! Transport-agnostic error for the device service.
//!
//! L1 carries no web-framework types, so the *kind* is the contract: the daemon
//! marshals each kind to its own wire shape (REST status code, Connect error
//! code). Mirrors the variants the daemon's command gate distinguishes, so the
//! device handler's marshal is a 1:1 of the legacy space-window marshal.

/// The kind of a [`DeviceError`]. The daemon maps each to a status/code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceErrorKind {
    /// Invalid request input (→ 400 / invalid_argument).
    BadArgs,
    /// The named entity does not exist (→ 404 / not_found).
    NotFound,
    /// The caller is not permitted (→ 403 / permission_denied).
    Forbidden,
    /// A required precondition is unmet, e.g. no space identity (→ 409 / aborted).
    Conflict,
    /// An internal failure (→ 500 / internal).
    Service,
}

/// Structured service error carrying a transport-agnostic [`DeviceErrorKind`] +
/// a message. The daemon's marshal layer maps `kind` to its status code.
#[derive(Debug, Clone)]
pub struct DeviceError {
    pub kind: DeviceErrorKind,
    pub message: String,
}

impl DeviceError {
    pub fn bad_args(msg: impl Into<String>) -> Self {
        Self { kind: DeviceErrorKind::BadArgs, message: msg.into() }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self { kind: DeviceErrorKind::NotFound, message: msg.into() }
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self { kind: DeviceErrorKind::Forbidden, message: msg.into() }
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self { kind: DeviceErrorKind::Conflict, message: msg.into() }
    }
    pub fn service(msg: impl Into<String>) -> Self {
        Self { kind: DeviceErrorKind::Service, message: msg.into() }
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for DeviceError {}
