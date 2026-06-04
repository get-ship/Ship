//! Single-source `agents.sessions` op — descriptor + dispatch via `ship_ops!`.
//! The live agent-session/dispatch index, served as an AgentsService method
//! (agents-arc E6: `sessions.list → agents.sessions`).

use super::wire::SessionsListResponse;

ship_ops! {
    SESSIONS_OPS;
    {
        name: "agents.sessions",
        desc: "All live PTY sessions (agent chats, pipeline runs, shells) — Mission Control data.",
        perm: "ship.sessions.read",
        args: &[],
        ret: SessionsListResponse,
        run: |_args, state, _files| {
            Ok(SessionsListResponse { items: state.pty_sessions().list() })
        },
    },
}
