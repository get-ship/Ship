//! Small std-only string utilities shared across layers.
//!
//! L0 leaf — no domain knowledge, no git/process/event deps. Originally
//! hosted by `ship_code::runner::observe::strip_ansi` and the byte-flavored
//! `pipeline_defs::log_stream::strip_ansi` near-duplicate; both call
//! sites now route through here so the ANSI escape grammar is defined once.

/// Strip ANSI CSI escape sequences from a `str`, returning the cleaned text.
///
/// Handles the two forms the rest of the workspace actually produces:
/// - CSI: `ESC [ <params> <final byte>`, where `<final byte>` is `@`-`~`
///   (the SGR colour-reset, cursor-position, etc. terminators).
/// - Single-character escapes (e.g. `ESC X`): the trailing byte is dropped
///   without examining it. Without this branch a stray `\x1bX` (used by
///   some harnesses as a status indicator) would survive the stripper and
///   pollute downstream SSE / log consumers.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if matches!(nc, '\x40'..='\x7E') {
                        break;
                    }
                }
                continue;
            }
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

/// Byte-flavored wrapper around [`strip_ansi`] for callers that read log
/// chunks off disk and want a `String` back. Invalid UTF-8 bytes are
/// lossy-converted (matches the prior `log_stream::strip_ansi` contract).
pub fn strip_ansi_bytes(input: &[u8]) -> String {
    let s = String::from_utf8_lossy(input);
    strip_ansi(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_csi_color_codes() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strip_ansi_passes_plain_text_through() {
        assert_eq!(strip_ansi("plain text\nline 2"), "plain text\nline 2");
    }

    #[test]
    fn strip_ansi_handles_non_csi_escape() {
        assert_eq!(strip_ansi("a\x1bXb"), "ab");
    }

    #[test]
    fn strip_ansi_bytes_handles_unicode() {
        let s = std::str::from_utf8(b"\x1b[31mcaf\xc3\xa9\x1b[0m").unwrap();
        assert_eq!(strip_ansi(s), "café");
    }

    #[test]
    fn strip_ansi_bytes_replaces_invalid_utf8() {
        let bytes: &[u8] = b"\x1b[31m\xe2\x28\x1b[0mrest";
        // invalid `\xe2\x28` becomes the U+FFFD replacement char
        assert_eq!(strip_ansi_bytes(bytes), "\u{fffd}(rest");
    }
}
