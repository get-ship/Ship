//! Shared byte-scan primitives for the compiler's textual passes.
//!
//! Several passes (parse, render, check, emit, lower) walk raw `.ts`/expr
//! source byte-by-byte. The lexical leaves — "is this byte an identifier
//! char?" — were re-defined in 8 places; this is their one home. They are
//! pure, stable character-class predicates with no pass-specific knowledge.
//!
//! (The `skip_*` scanners are NOT here yet: their copies diverge in
//! string/comment-awareness and bounded-vs-unbounded signatures, so unifying
//! them is a separate, per-caller-verified step — see #161.)

/// A JavaScript identifier start byte: ASCII letter, `_`, or `$`.
#[inline]
pub(crate) fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}

/// A JavaScript identifier continuation byte: an [`is_ident_start`] byte or a digit.
#[inline]
pub(crate) fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}
