//! Tests for the chat view context + template rendering.

use super::*;
use crate::views::chat_state::{ChatMessage, Sender};
use crate::views::templates;

fn msg(id: &str, sender: Sender, content: &str, kind: &str) -> ChatMessage {
    ChatMessage {
        id: id.into(),
        sender,
        content: content.into(),
        detail: None,
        timestamp: "12:00".into(),
        event_type: kind.into(),
        workspace_id: "ws-1".into(),
    }
}

fn sample_ctx(messages: Vec<ChatMessage>) -> ChatCtx {
    ChatCtx {
        workspace_id: "ws-1".into(),
        total: messages.len(),
        messages,
    }
}

#[test]
fn shell_renders_messages_and_composer() {
    let ctx = sample_ctx(vec![
        msg("1", Sender::User, "hello", "studio.message"),
        msg("2", Sender::Agent, "working on it", "session.progress"),
    ]);
    let html = templates::render("chat/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("id=\"chat-root\""));
    assert!(html.contains("id=\"chat-messages\""));
    assert!(html.contains("id=\"chat-composer\""));
    assert!(html.contains("hello"));
    assert!(html.contains("working on it"));
}

#[test]
fn messages_partial_renders_bubbles_with_sender_attr() {
    let ctx = sample_ctx(vec![
        msg("1", Sender::User, "ping", "studio.message"),
        msg("2", Sender::System, "Session started", "session.started"),
    ]);
    let html = templates::render("chat/partials/messages.html.j2", &ctx).expect("render");
    assert!(html.contains("data-sender=\"user\""));
    assert!(html.contains("data-sender=\"system\""));
    assert!(html.contains("Session started"));
}

#[test]
fn empty_state_when_no_messages() {
    let ctx = sample_ctx(Vec::new());
    let html = templates::render("chat/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("No messages yet"), "empty state: {html}");
}

#[test]
fn composer_posts_via_chat_send_endpoint() {
    let ctx = sample_ctx(Vec::new());
    let html = templates::render("chat/partials/composer.html.j2", &ctx).expect("render");
    assert!(html.contains("chat-composer"));
    // Send hooks into the typed URL macro so the wire contract is locked.
    assert!(
        html.contains("/api/runtime/view/chat/send"),
        "composer should POST to chat/send: {html}"
    );
}

#[test]
fn message_detail_renders_escalation_label() {
    let mut m = msg("1", Sender::Agent, "needs input", "session.progress");
    m.detail = Some("[needs_human]".into());
    let ctx = sample_ctx(vec![m]);
    let html = templates::render("chat/partials/messages.html.j2", &ctx).expect("render");
    assert!(html.contains("[needs_human]"));
}
