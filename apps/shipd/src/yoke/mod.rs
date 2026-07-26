//! shipd ↔ Yoke seam (L2 host → L1 embeddable harness).
//!
//! Yoke is a **headless rig** — it does not depend on Bridge/fleet.
//! shipd embeds it in-process and supplies [`gateway::ShipdGateway`] + kernel sink.
//!
//! - [`gateway`] — answers Yoke `Gateway` calls via the in-process ops router
//! - [`embed`] — **product path**: in-process `YokeProgram` + durable journal
//! - [`yoke_rpc_client`] — optional out-of-process fleet (`bridge serve`) path;
//!   not required for native shipd sessions
//!
//! Boot wiring (`daemon::lifecycle`) hands a `YokeRunFn` that defaults to embed.

pub mod embed;
pub mod gateway;
pub mod yoke_rpc_client;
