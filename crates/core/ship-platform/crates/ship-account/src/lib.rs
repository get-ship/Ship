//! `ship-account` — Ship Cloud identity over substrate ports (L1).
//!
//! Login / logout / state / key for the daemon's Ship Cloud account, plus the
//! cloud-gateway resolution that identity exchange depends on. Four seams, all
//! substrate ports — **zero raw `fs`** in this crate:
//! - token at rest → `ship_substrate::config_doc` secret-doc (chmod 600,
//!   atomic write, parent-create, idempotent clear)
//! - `account.*` audit → `SpaceContext.kernel()` (KernelLog port, Account-scoped
//!   → `account.db`), NOT the legacy `SqliteEventStore::append` bypass
//! - gateway exchange / me / revoke → `ship-gateway-client`
//! - gateway base URL + stored-key read (`gateway` module) → `config_doc` over
//!   `settings.jsonc:cloud.base_url` and the single owner doc `account.jsonc`
//!   (S5: the legacy `config::{credentials,gateway}` tier folded in here)
//!
//! Account identity is process-global, NOT space-partitioned: no `DataStore`,
//! no per-space table. The owner-binding (`daemon.owner.bound/unbound`) is an
//! injected callback so this crate stays decoupled from the `ship-platform`
//! parent (which depends on it); the daemon wires `auth::attest` at construction.

pub mod error;
pub mod events;
pub mod event_registrations;
pub mod gateway;
pub mod service;
pub mod store;
pub mod types;
pub mod wire;

pub use error::AccountError;
pub use gateway::{resolve_gateway_base_url, DEFAULT_GATEWAY};
pub use service::{AccountService, OwnerBinder, StartLoginResponse};
pub use store::{AccountTokenStore, FileTokenStore};
pub use types::{AccountState, AccountToken, MeView};
pub use wire::AccountWire;
