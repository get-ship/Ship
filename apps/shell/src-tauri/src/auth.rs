//! Native C3 escalation gate.
//!
//! The webview holds the operator token (C0–C2, capped by the daemon's
//! blast-tier gate). Crossing to C3 (`agent.grant`, account/billing/permissions)
//! requires a *live human gate* that the webview cannot satisfy on its own. This
//! command is that gate: a native dialog the operator must approve.
//!
//! Scope boundary (honest about what's built): this implements the **human
//! approval half** — the native confirmation that a C3 request demands. Minting
//! the short-lived C3 credential against the daemon and performing the elevated
//! call is gated on #86 (daemon-token downgrade + C3 step-up endpoint); it is NOT
//! implemented here and is not faked. The decision returned tells the caller
//! whether the operator approved, nothing more.

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Operator's response to an escalation prompt.
#[derive(Debug, Clone, Serialize)]
pub struct EscalationDecision {
    /// True iff the operator approved at the native prompt.
    pub approved: bool,
}

/// Present the native C3 approval prompt for `reason` and return the operator's
/// decision. The elevated credential never enters the webview; once #86 lands,
/// an approved decision is what authorizes the core to mint + use the C3 step-up.
#[tauri::command]
pub async fn request_escalation(app: AppHandle, reason: String) -> EscalationDecision {
    let approved = app
        .dialog()
        .message(format!(
            "An operation requires elevated (C3) operator approval:\n\n{reason}"
        ))
        .title("Operator approval required")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Approve".into(),
            "Deny".into(),
        ))
        .blocking_show();
    EscalationDecision { approved }
}
