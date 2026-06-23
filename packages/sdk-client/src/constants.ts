// @ship/sdk-client/constants — client-side URL constants for the deck
// serve-scheme. Lives in its own module so both `index.ts` and
// `mux-client.ts` can import without forming a cycle: the constants have no
// runtime deps on the rest of the SDK surface, so a circular import through
// here is impossible.
//
// `DECK_PREFIX` and `DECK_API_ASSETS` are mirrored on the Rust side at
// `deck_core::routes::{DECK_PREFIX, DECK_API_ASSETS}`. The
// `ts_constants_pinned_to_rust` test in `crates/deck/deck-core/src/routes.rs`
// reads this file and asserts the literals match so the two halves cannot
// drift.

/** URL prefix for the serve-scheme surfaces deck owns (`/_deck/*`). */
export const DECK_PREFIX = '/_deck';

/** URL prefix for the daemon-served design-token / theme / shared CSS surface. */
export const DECK_API_ASSETS = '/api/assets';
