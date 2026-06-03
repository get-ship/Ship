//! `ship-account` — Ship Cloud identity over substrate ports (L1).
//!
//! Login / logout / state / key for the daemon's Ship Cloud account. Three
//! seams, all substrate ports — **zero raw `fs`** in this crate:
//! - token at rest → `ship_substrate::config_doc` secret-doc (chmod 600,
//!   atomic write, parent-create, idempotent clear)
//! - `account.*` audit → `SpaceContext.kernel()` (KernelLog port, Account-scoped
//!   → `account.db`), NOT the legacy `SqliteEventStore::append` bypass
//! - gateway exchange / me / revoke → `ship-gateway-client`
//!
//! Account identity is process-global, NOT space-partitioned: no `DataStore`,
//! no per-space table. The owner-binding (`daemon.owner.bound/unbound`) is an
//! injected callback so this crate stays decoupled from the `ship-platform`
//! parent (which depends on it); the daemon wires `auth::attest` at construction.

pub mod error;
pub mod events;
pub mod service;
pub mod store;
pub mod types;
pub mod wire;

pub use error::AccountError;
pub use service::{AccountService, OwnerBinder, StartLoginResponse};
pub use store::{AccountTokenStore, FileTokenStore};
pub use types::{AccountState, AccountToken, MeView};
pub use wire::AccountWire;
