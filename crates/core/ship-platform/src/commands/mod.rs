//! Slash-command registry for the command bar (cmdk).
//!
//! Storing the canonical list server-side lets new modes (e.g. edit, search-share)
//! grow without palette edits in the SUI islands. The frontend fetches
//! `/api/v1/commands` and renders whatever the daemon advertises.
//!
//! Built-in commands are seeded at boot via [`seed_builtins`]. Additional
//! commands can be added at runtime through [`register`] — the seam we'll
//! grow into a manifest-export later.

use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

/// Whether the command is handled inside the bar (mode flip, close) or
/// dispatches to a built-in handler (navigate, lint, end). Distinguishes the
/// frontend's behavior — the daemon doesn't run handlers itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CommandHandlerKind {
    /// Flips the bar into a different mode (e.g. `/new` → create).
    Mode,
    /// Navigates to a path (e.g. `/chats` → /chats).
    Navigate,
    /// Calls a daemon-side endpoint (e.g. `/lint` → POST /api/v1/lint/view).
    Endpoint,
    /// Pure-bar behavior (close, end session).
    Local,
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CommandEntry {
    /// Slash key without the leading `/` (e.g. `"new"`).
    pub key: String,
    /// Display label shown in the palette (e.g. `"/new"`).
    pub label: String,
    /// One-line hint shown alongside the label.
    pub hint: String,
    pub handler_kind: CommandHandlerKind,
}

static REGISTRY: OnceLock<RwLock<Vec<CommandEntry>>> = OnceLock::new();

fn registry() -> &'static RwLock<Vec<CommandEntry>> {
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// Idempotent: seeds the built-in commands. Re-seeding leaves duplicates
/// out — keyed on `key`.
pub fn seed_builtins() {
    let builtins = [
        ("new",      "/new",      "Scaffold a view, skill, agent, …", CommandHandlerKind::Mode),
        ("chats",    "/chats",    "Open Mission Control",             CommandHandlerKind::Navigate),
        ("settings", "/settings", "Jump to settings",                 CommandHandlerKind::Navigate),
        ("lint",     "/lint",     "Run lint on the current view",     CommandHandlerKind::Endpoint),
        ("end",      "/end",      "End the current session",          CommandHandlerKind::Local),
        ("close",    "/close",    "Close the command bar",            CommandHandlerKind::Local),
    ];
    let mut guard = registry().write().expect("commands registry poisoned");
    for (key, label, hint, kind) in builtins {
        if !guard.iter().any(|c| c.key == key) {
            guard.push(CommandEntry {
                key: key.to_string(),
                label: label.to_string(),
                hint: hint.to_string(),
                handler_kind: kind,
            });
        }
    }
}

/// Insert (or upsert by key) a command at runtime.
pub fn register(entry: CommandEntry) {
    let mut guard = registry().write().expect("commands registry poisoned");
    if let Some(existing) = guard.iter_mut().find(|c| c.key == entry.key) {
        *existing = entry;
    } else {
        guard.push(entry);
    }
}

/// Snapshot of the current registry. Stable order: seeded first, then
/// runtime additions in insertion order.
pub fn list() -> Vec<CommandEntry> {
    registry().read().expect("commands registry poisoned").clone()
}

#[cfg(test)]
pub fn reset_for_test() {
    registry().write().expect("commands registry poisoned").clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The registry is process-global, so tests must run serially or they
    // stomp each other's reset/seed sequence.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn seed_is_idempotent() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        seed_builtins();
        seed_builtins();
        let cmds = list();
        let new_count = cmds.iter().filter(|c| c.key == "new").count();
        assert_eq!(new_count, 1);
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn register_upserts_by_key() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        seed_builtins();
        register(CommandEntry {
            key: "lint".to_string(),
            label: "/lint".to_string(),
            hint: "updated hint".to_string(),
            handler_kind: CommandHandlerKind::Endpoint,
        });
        let cmds = list();
        let lint = cmds.iter().find(|c| c.key == "lint").expect("lint present");
        assert_eq!(lint.hint, "updated hint");
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn register_appends_new_keys_after_seed() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        seed_builtins();
        register(CommandEntry {
            key: "share".to_string(),
            label: "/share".to_string(),
            hint: "share this view".to_string(),
            handler_kind: CommandHandlerKind::Endpoint,
        });
        let cmds = list();
        assert_eq!(cmds.len(), 7);
        assert_eq!(cmds.last().unwrap().key, "share");
    }
}
