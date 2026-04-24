//! Chat domain service. Read: `chat.state(workspace_id?)` via the
//! registry. Write: `POST /api/chat/send` emits a `studio.message`
//! kernel event so the composer is one daemon round-trip.

pub mod send;

pub use send::send_message;
