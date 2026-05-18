//! Ship Cloud account — daemon-side state and orchestration.
//!
//! ## What lives here
//!
//! - **Account token store** ([`store`]) — the Better Auth API key minted
//!   by `ship login`, stored under `~/.ship/machine/daemon/account.jsonc`
//!   (mode 0600). Owned by the daemon process. NOT in the shared CLI
//!   credentials file.
//! - **Pending logins** ([`PendingLogins`]) — in-memory map of in-flight
//!   PKCE flows. CLI starts a login → daemon registers a state, stores
//!   the verifier, returns the browser URL. CLI gets the code from the
//!   browser callback → daemon exchanges code+verifier for the API key.
//! - **Account service** ([`AccountService`]) — high-level operations
//!   (start_login / complete_login / get_me / logout) that daemon HTTP
//!   routes call into.
//!
//! ## What does NOT live here
//!
//! - Daemon-owner binding (the `auth.attest::set_daemon_owner` event +
//!   the daemon-bound-to-user record). That's a separate concern living
//!   under `auth/attest`. Account state is "what gateway API key are we
//!   holding"; daemon-owner is "which user does this machine belong to."
//!   The two are related but separately persisted.
//! - The gateway-side OAuth flow itself. The CLI's browser callback
//!   server still runs in the CLI process — daemon supplies the PKCE
//!   challenge and exchanges the resulting code, but the OAuth dance is
//!   between the user's browser and the gateway.

pub mod service;
pub mod store;
pub mod types;

pub use service::AccountService;
pub use store::{AccountTokenStore, FileTokenStore};
pub use types::{AccountState, AccountToken, MeView};
