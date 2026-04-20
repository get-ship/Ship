// Canonical EventEnvelope and event-type string constants live in ./generated,
// produced by `cargo xtask gen-types` from crates/core/ship-types. Cloud is a
// sync target for the Rust runtime — it does not invent its own event
// taxonomy. Import constants directly from '@ship-cloud/types' (which
// re-exports from ./generated) when writing projection handlers.

export type { EventEnvelope } from './generated.js';
