//! Phase 6 — CSS scoping + `:host` rewrite + token resolution.
//!
//! Reuses the algorithm from `apps/shipd/src/views/sfc/style.rs`:
//! - Prefix every top-level selector with `[data-component="<tag>"]`
//! - Rewrite `:host` → `[data-component="<tag>"]` (compound preserved)
//! - Rewrite `${token.path}` → `var(--ship-token-path)` after validating
//!   the token exists in the configured token source
//! - Honor `<style global>` (skip scoping)
//! - Honor `<style src="./x.css">` (read file, prepend)
//!
//! Validation errors (unknown tokens, malformed selectors) emitted as
//! diagnostics.
