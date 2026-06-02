//! Suspicious-Unicode classification table for the content scanner.
//!
//! The `SUSPICIOUS_RANGES` data and the `classify_char` lookup it backs are
//! split out from the scan loop in `scan.rs` to keep each file under the cap.

/// Severity of a scan finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// No legitimate use in prompt files. Block by default.
    Critical,
    /// Suspicious — common copy-paste debris but can hide instructions.
    Warning,
    /// Unusual but mostly harmless (e.g. non-breaking space).
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Suspicious Unicode character ranges.
///
/// Each entry: (start, end_inclusive, severity, category, description).
const SUSPICIOUS_RANGES: &[(u32, u32, Severity, &str, &str)] = &[
    // ── Critical: no legitimate use in prompt files ──
    // Unicode tag characters — invisible ASCII mapping
    (
        0xE0001,
        0xE007F,
        Severity::Critical,
        "tag-character",
        "Unicode tag character (invisible ASCII mapping)",
    ),
    // Bidirectional override characters
    (
        0x202A,
        0x202A,
        Severity::Critical,
        "bidi-override",
        "Left-to-right embedding (LRE)",
    ),
    (
        0x202B,
        0x202B,
        Severity::Critical,
        "bidi-override",
        "Right-to-left embedding (RLE)",
    ),
    (
        0x202C,
        0x202C,
        Severity::Critical,
        "bidi-override",
        "Pop directional formatting (PDF)",
    ),
    (
        0x202D,
        0x202D,
        Severity::Critical,
        "bidi-override",
        "Left-to-right override (LRO)",
    ),
    (
        0x202E,
        0x202E,
        Severity::Critical,
        "bidi-override",
        "Right-to-left override (RLO)",
    ),
    (
        0x2066,
        0x2066,
        Severity::Critical,
        "bidi-override",
        "Left-to-right isolate (LRI)",
    ),
    (
        0x2067,
        0x2067,
        Severity::Critical,
        "bidi-override",
        "Right-to-left isolate (RLI)",
    ),
    (
        0x2068,
        0x2068,
        Severity::Critical,
        "bidi-override",
        "First strong isolate (FSI)",
    ),
    (
        0x2069,
        0x2069,
        Severity::Critical,
        "bidi-override",
        "Pop directional isolate (PDI)",
    ),
    // Variation selectors (SMP) — Glassworm attack vector
    (
        0xE0100,
        0xE01EF,
        Severity::Critical,
        "variation-selector",
        "Variation selector (SMP) — no legitimate use in prompt files",
    ),
    // ── Warning: suspicious but sometimes benign ──
    (
        0x200B,
        0x200B,
        Severity::Warning,
        "zero-width",
        "Zero-width space",
    ),
    (
        0x200C,
        0x200C,
        Severity::Warning,
        "zero-width",
        "Zero-width non-joiner (ZWNJ)",
    ),
    (
        0x200D,
        0x200D,
        Severity::Warning,
        "zero-width",
        "Zero-width joiner (ZWJ)",
    ),
    (
        0x2060,
        0x2060,
        Severity::Warning,
        "zero-width",
        "Word joiner",
    ),
    (
        0xFE00,
        0xFE0D,
        Severity::Warning,
        "variation-selector",
        "Variation selector (CJK)",
    ),
    (
        0xFE0E,
        0xFE0E,
        Severity::Warning,
        "variation-selector",
        "Text presentation selector",
    ),
    (
        0x00AD,
        0x00AD,
        Severity::Warning,
        "invisible-formatting",
        "Soft hyphen",
    ),
    (
        0x200E,
        0x200E,
        Severity::Warning,
        "bidi-mark",
        "Left-to-right mark (LRM)",
    ),
    (
        0x200F,
        0x200F,
        Severity::Warning,
        "bidi-mark",
        "Right-to-left mark (RLM)",
    ),
    (
        0x061C,
        0x061C,
        Severity::Warning,
        "bidi-mark",
        "Arabic letter mark (ALM)",
    ),
    (
        0x2061,
        0x2061,
        Severity::Warning,
        "invisible-formatting",
        "Function application",
    ),
    (
        0x2062,
        0x2062,
        Severity::Warning,
        "invisible-formatting",
        "Invisible times",
    ),
    (
        0x2063,
        0x2063,
        Severity::Warning,
        "invisible-formatting",
        "Invisible separator",
    ),
    (
        0x2064,
        0x2064,
        Severity::Warning,
        "invisible-formatting",
        "Invisible plus",
    ),
    (
        0xFFF9,
        0xFFF9,
        Severity::Warning,
        "annotation-marker",
        "Interlinear annotation anchor",
    ),
    (
        0xFFFA,
        0xFFFA,
        Severity::Warning,
        "annotation-marker",
        "Interlinear annotation separator",
    ),
    (
        0xFFFB,
        0xFFFB,
        Severity::Warning,
        "annotation-marker",
        "Interlinear annotation terminator",
    ),
    (
        0x206A,
        0x206F,
        Severity::Warning,
        "deprecated-formatting",
        "Deprecated formatting character",
    ),
    // ── Info: unusual whitespace ──
    (
        0xFE0F,
        0xFE0F,
        Severity::Info,
        "variation-selector",
        "Emoji presentation selector",
    ),
    (
        0x00A0,
        0x00A0,
        Severity::Info,
        "unusual-whitespace",
        "Non-breaking space",
    ),
    (
        0x2000,
        0x200A,
        Severity::Info,
        "unusual-whitespace",
        "Unicode whitespace character",
    ),
    (
        0x205F,
        0x205F,
        Severity::Info,
        "unusual-whitespace",
        "Medium mathematical space",
    ),
    (
        0x3000,
        0x3000,
        Severity::Info,
        "unusual-whitespace",
        "Ideographic space",
    ),
    (
        0x180E,
        0x180E,
        Severity::Info,
        "unusual-whitespace",
        "Mongolian vowel separator",
    ),
];

/// Classify a codepoint against the suspicious ranges.
pub(super) fn classify_char(cp: u32) -> Option<(Severity, &'static str, &'static str)> {
    for &(start, end, sev, cat, desc) in SUSPICIOUS_RANGES {
        if cp >= start && cp <= end {
            return Some((sev, cat, desc));
        }
    }
    None
}
