//! Single-source `sessions.*` ops — descriptors + dispatch via `ship_ops!`.

use super::wire::SessionsListResponse;

ship_ops! {
    SESSIONS_OPS;
    {
        name: "sessions.list",
        desc: "All live PTY sessions (agent chats, pipeline runs, shells) — Mission Control data.",
        perm: "ship.sessions.read",
        args: &[],
        ret: SessionsListResponse,
        run: |_args, state, _files| {
            Ok(SessionsListResponse { items: state.pty_sessions().list() })
        },
    },
}
