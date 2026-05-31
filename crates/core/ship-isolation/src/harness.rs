//! Per-harness sandbox knowledge.
//!
//! Tier 1 splits into two strategies because the reality across harnesses
//! is split:
//!
//!   - `HarnessNative` — the harness sandboxes itself (claude `--sandbox`,
//!     codex Seatbelt on macOS, gemini `--yolo`). Ship passes a CLI flag
//!     (if any) and trusts the harness.
//!   - `SrtWrapper` — we wrap with `@anthropic-ai/sandbox-runtime` when
//!     the harness has no native sandbox (opencode, cursor-agent, codex
//!     on Linux).
//!
//! This module is the lookup table: given (harness id, host OS), what's
//! the right Tier 1 mechanism? Recommender + supervisor consult it; the
//! harness adapter trait stays clean (adapters know how to launch, not
//! how to sandbox).

use serde::{Deserialize, Serialize};

use crate::capabilities::HostOs;

/// How a particular (harness, OS) pair should be sandboxed at Tier 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarnessSandboxMechanism {
    /// Harness sandboxes itself. We pass `flag` if `Some` (e.g.
    /// `--sandbox` for claude, `--yolo` for gemini); otherwise the
    /// harness sandboxes by default and there's no flag to pass
    /// (codex on macOS via Seatbelt — handled internally).
    Native {
        /// Optional CLI flag we add to the launch command.
        flag: Option<&'static str>,
        /// Human-readable label for UI/CLI surfacing.
        label: &'static str,
    },
    /// Wrap with srt. The supervisor handles bun + srt setup.
    SrtWrapper {
        /// Human-readable label.
        label: &'static str,
    },
    /// This harness has no Tier 1 mechanism on this OS — recommender
    /// must fall back to Tier 0 or escalate to Tier 2+.
    Unavailable {
        /// Why — surfaces in `ship isolation status` and the UI.
        reason: &'static str,
    },
}

impl HarnessSandboxMechanism {
    /// Default per-allowlist hostnames the harness will reach when it
    /// runs. The recommender unions these with the user's allowlist
    /// when building a SandboxPolicy.
    pub fn default_allowlist(harness_id: &str) -> Vec<&'static str> {
        match harness_id {
            "claude" | "claude-code" => vec!["api.anthropic.com", "console.anthropic.com"],
            "codex" => vec!["api.openai.com", "chatgpt.com"],
            "gemini" => vec!["generativelanguage.googleapis.com", "aiplatform.googleapis.com"],
            "opencode" => vec![
                "api.anthropic.com",
                "api.openai.com",
                "openrouter.ai",
            ],
            "cursor" | "cursor-agent" => vec!["api2.cursor.sh", "cursor.com"],
            _ => Vec::new(),
        }
    }
}

/// The full per-harness Tier 1 table. Pure function of (harness, OS);
/// no I/O. Strategy.md §3.1 is the spec; this is the implementation.
pub fn tier1_mechanism_for(harness_id: &str, os: HostOs) -> HarnessSandboxMechanism {
    use HarnessSandboxMechanism::*;
    match (harness_id, os) {
        // ── claude — uses srt internally on every OS we support.
        // The `--sandbox` flag activates its own machinery; we don't
        // wrap externally (double-wrap is fragile per strategy.md).
        ("claude" | "claude-code", HostOs::MacosArm | HostOs::MacosIntel | HostOs::Linux | HostOs::Wsl2) => {
            Native {
                flag: Some("--sandbox"),
                label: "Sandboxed by Claude",
            }
        }
        ("claude" | "claude-code", HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "Claude's native sandbox doesn't ship for Windows",
        },

        // ── codex — Seatbelt on macOS, no native sandbox on Linux.
        // Codex applies Seatbelt internally; no flag to pass.
        ("codex", HostOs::MacosArm | HostOs::MacosIntel) => Native {
            flag: None,
            label: "Sandboxed by Codex (Seatbelt)",
        },
        ("codex", HostOs::Linux | HostOs::Wsl2) => SrtWrapper {
            label: "Sandboxed by Ship (srt)",
        },
        ("codex", HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "Codex has no Linux/Windows sandbox; install srt for SrtWrapper",
        },

        // ── gemini — `--yolo` is the closest analogue (relaxed
        // permission gate; relies on harness sandboxing internally).
        ("gemini", HostOs::MacosArm | HostOs::MacosIntel | HostOs::Linux | HostOs::Wsl2) => Native {
            flag: Some("--yolo"),
            label: "Sandboxed by Gemini",
        },
        ("gemini", HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "Gemini's --yolo sandbox doesn't ship for Windows",
        },

        // ── opencode — no native sandbox; always srt.
        ("opencode", HostOs::MacosArm | HostOs::MacosIntel | HostOs::Linux | HostOs::Wsl2) => {
            SrtWrapper {
                label: "Sandboxed by Ship (srt)",
            }
        }
        ("opencode", HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "srt has no Windows backend",
        },

        // ── cursor-agent — no native sandbox; always srt.
        ("cursor" | "cursor-agent", HostOs::MacosArm | HostOs::MacosIntel | HostOs::Linux | HostOs::Wsl2) => {
            SrtWrapper {
                label: "Sandboxed by Ship (srt)",
            }
        }
        ("cursor" | "cursor-agent", HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "srt has no Windows backend",
        },

        // ── unknown harness — assume no native sandbox; fall through to srt.
        (_, HostOs::MacosArm | HostOs::MacosIntel | HostOs::Linux | HostOs::Wsl2) => SrtWrapper {
            label: "Sandboxed by Ship (srt)",
        },
        (_, HostOs::Windows | HostOs::Unknown) => Unavailable {
            reason: "no Tier 1 mechanism available for this (harness, OS)",
        },
    }
}

/// Convenience: does this harness on this OS have ANY Tier 1 mechanism?
pub fn has_tier1(harness_id: &str, os: HostOs) -> bool {
    !matches!(
        tier1_mechanism_for(harness_id, os),
        HarnessSandboxMechanism::Unavailable { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_uses_native_sandbox_with_flag_on_macos_arm() {
        let m = tier1_mechanism_for("claude", HostOs::MacosArm);
        match m {
            HarnessSandboxMechanism::Native { flag, .. } => assert_eq!(flag, Some("--sandbox")),
            _ => panic!("expected Native"),
        }
    }

    #[test]
    fn claude_code_alias_maps_to_same_mechanism() {
        let a = tier1_mechanism_for("claude", HostOs::Linux);
        let b = tier1_mechanism_for("claude-code", HostOs::Linux);
        assert_eq!(a, b);
    }

    #[test]
    fn codex_native_on_macos_no_flag() {
        let m = tier1_mechanism_for("codex", HostOs::MacosArm);
        match m {
            HarnessSandboxMechanism::Native { flag, .. } => assert!(flag.is_none()),
            _ => panic!("expected Native"),
        }
    }

    #[test]
    fn codex_falls_through_to_srt_on_linux() {
        let m = tier1_mechanism_for("codex", HostOs::Linux);
        assert!(matches!(m, HarnessSandboxMechanism::SrtWrapper { .. }));
    }

    #[test]
    fn gemini_native_uses_yolo() {
        let m = tier1_mechanism_for("gemini", HostOs::MacosArm);
        match m {
            HarnessSandboxMechanism::Native { flag, .. } => assert_eq!(flag, Some("--yolo")),
            _ => panic!("expected Native"),
        }
    }

    #[test]
    fn opencode_always_srt() {
        for os in [HostOs::MacosArm, HostOs::Linux, HostOs::Wsl2] {
            let m = tier1_mechanism_for("opencode", os);
            assert!(matches!(m, HarnessSandboxMechanism::SrtWrapper { .. }), "os={os:?}");
        }
    }

    #[test]
    fn windows_is_unavailable_for_all_harnesses() {
        for h in ["claude", "codex", "gemini", "opencode", "cursor"] {
            let m = tier1_mechanism_for(h, HostOs::Windows);
            assert!(
                matches!(m, HarnessSandboxMechanism::Unavailable { .. }),
                "harness={h}"
            );
        }
    }

    #[test]
    fn unknown_harness_falls_through_to_srt() {
        let m = tier1_mechanism_for("totally-made-up", HostOs::Linux);
        assert!(matches!(m, HarnessSandboxMechanism::SrtWrapper { .. }));
    }

    #[test]
    fn has_tier1_reports_availability_correctly() {
        assert!(has_tier1("claude", HostOs::MacosArm));
        assert!(has_tier1("opencode", HostOs::Linux));
        assert!(!has_tier1("claude", HostOs::Windows));
        assert!(!has_tier1("claude", HostOs::Unknown));
    }

    #[test]
    fn default_allowlist_has_known_harness_endpoints() {
        assert!(
            HarnessSandboxMechanism::default_allowlist("claude")
                .contains(&"api.anthropic.com")
        );
        assert!(
            HarnessSandboxMechanism::default_allowlist("codex")
                .contains(&"api.openai.com")
        );
        assert!(
            HarnessSandboxMechanism::default_allowlist("gemini")
                .iter()
                .any(|h| h.contains("googleapis"))
        );
    }

    #[test]
    fn unknown_harness_has_empty_allowlist() {
        assert!(HarnessSandboxMechanism::default_allowlist("totally-unknown").is_empty());
    }
}
