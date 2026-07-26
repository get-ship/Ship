//! Dual-LSP seam — semantic code intelligence over BOTH of Ship's languages.
//!
//! The agent runtime consumes language services two ways:
//!   1. Authoring metadata (hover, definition, …) surfaced as a tool the model
//!      can call when reading unfamiliar code.
//!   2. The verify gate — after an edit, run diagnostics; a new error means the
//!      edit is `UNVERIFIED`, caught *before* execution.
//!
//! Routing is total: every `path` produces a `Backend`, even when no language
//! service is wired for the extension. The seam currently has no real tsgo or
//! SUI-LSP wired — the stubs return empty/canned data, clearly marked as where
//! the real backends land.
//!
//!   TypeScript (`.ts`/`.tsx`)  -> `TsgoStub`    (real: tsgo)
//!   SUI templates (`.sui`)     -> `SuiLspStub`  (real: the sui repo's sui-lsp crate, post-carve)
//!   anything else              -> `TsgoStub`    (permissive default; a real
//!                                                 wiring will narrow this with
//!                                                 a registry, not a panic)

use thiserror::Error;

use crate::spine::SpineError;

/// Severity of a diagnostic. The verify gate treats only `Error` as fatal;
/// `Warning` and below are reported but do not refuse the edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    #[allow(dead_code)] // exercised by tests; not by the demo binary
    Warning,
}

/// A single diagnostic emitted by a language service. `line` is 1-indexed to
/// match the LSP convention; column, when needed, is in `message` for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: u32,
}

/// Errors that a real LSP backend may surface to the agent. The stubs do not
/// produce these; the type is here so the trait signature does not need to
/// change when the real backends land.
#[derive(Debug, Error)]
#[allow(dead_code)] // forward-declared for the real backends; stubs do not produce it
pub enum LspError {
    #[error("LSP backend failed: {0}")]
    Backend(String),
}

/// A language-service backend. Stubs implement this; the real tsgo and
/// the sui-lsp crate (in the sui repo, post-carve) implement it too, with
/// the SAME signature, so the verify gate and the metadata tools share
pub trait LspBackend {
    /// Return all current diagnostics for `src` (the on-disk contents of
    /// `path`). An empty vec means clean.
    fn diagnostics(&self, path: &str, src: &str) -> Vec<Diagnostic>;
    /// Hover text at `line`/`col` (1-indexed), or `None` if the position is
    /// not on a symbol.
    #[allow(dead_code)] // exercised by tests and future hover-call wiring; not the demo binary
    fn hover(&self, path: &str, src: &str, line: u32, col: u32) -> Option<String>;
}
/// Discriminant for `Backend`. Tests assert on `kind()`, not on the concrete
/// stub — the real wiring may swap the inner type without changing `kind()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // test-only identity surface; the binary routes via `verify_edit`
pub enum BackendKind {
    TypeScript,
    Sui,
}

/// A routed backend. The two variants carry the concrete stubs so a caller can
/// own one and use it directly; the `LspBackend` impl below dispatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Ts(TsgoStub),
    Sui(SuiLspStub),
}

impl Backend {
    /// The kind of this backend. Stable across real-wiring swaps.
    #[allow(dead_code)] // test-only identity accessor
    pub fn kind(&self) -> BackendKind {
        match self {
            Backend::Ts(_) => BackendKind::TypeScript,
            Backend::Sui(_) => BackendKind::Sui,
        }
    }
}

impl LspBackend for Backend {
    fn diagnostics(&self, path: &str, src: &str) -> Vec<Diagnostic> {
        match self {
            Backend::Ts(s) => s.diagnostics(path, src),
            Backend::Sui(s) => s.diagnostics(path, src),
        }
    }
    fn hover(&self, path: &str, src: &str, line: u32, col: u32) -> Option<String> {
        match self {
            Backend::Ts(s) => s.hover(path, src, line, col),
            Backend::Sui(s) => s.hover(path, src, line, col),
        }
    }
}

/// Stub for tsgo. Where the real tsgo adapter lands: replace this struct with
/// one that holds the tsgo handle and implements the trait against tsgo's
/// real `publishDiagnostics` and `hover` responses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TsgoStub;

impl LspBackend for TsgoStub {
    fn diagnostics(&self, _path: &str, _src: &str) -> Vec<Diagnostic> {
        // SEAM: returns clean until tsgo is wired.
        Vec::new()
    }
    fn hover(&self, _path: &str, _src: &str, _line: u32, _col: u32) -> Option<String> {
        // SEAM: returns nothing until tsgo is wired.
        None
    }
}

/// Stub for the SUI language server (the sui-lsp crate in the sui repo,
/// post-carve). Where the real adapter lands: replace this with a struct
/// that talks to the SUI LSP over its protocol.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SuiLspStub;

impl LspBackend for SuiLspStub {
    fn diagnostics(&self, _path: &str, _src: &str) -> Vec<Diagnostic> {
        // SEAM: returns clean until the SUI LSP is wired.
        Vec::new()
    }
    fn hover(&self, _path: &str, _src: &str, _line: u32, _col: u32) -> Option<String> {
        // SEAM: returns nothing until the SUI LSP is wired.
        None
    }
}

/// Route a file path to its backend. Total: every input yields a `Backend`.
/// Extensions are matched case-insensitively on the final path segment.
pub fn backend_for(path: &str) -> Backend {
    // Take the extension of the final segment; paths may carry directories
    // (e.g. `views/foo/bar.sui`).
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("ts") | Some("tsx") => Backend::Ts(TsgoStub),
        Some("sui") => Backend::Sui(SuiLspStub),
        // Permissive default: unknown extensions fall through to the TS
        // backend rather than refusing to verify. The real wiring will narrow
        // this against a known-extension registry; the stub does not pretend
        // to know.
        _ => Backend::Ts(TsgoStub),
    }
}

/// Run the post-edit verify gate: route the path, run diagnostics, and refuse
/// the edit if any error-severity diagnostic is reported. The spine wraps this
/// for code edits, using the SAME engine that serves authoring metadata.
pub fn verify_edit(path: &str, src: &str) -> Result<(), SpineError> {
    verify_edit_with(&backend_for(path), path, src)
}

/// The testable core of `verify_edit`: takes the backend explicitly so tests
/// can inject one that reports an error vs. a clean one, without hardcoding
/// tsgo output. The public `verify_edit` is a one-line wrapper around this.
pub fn verify_edit_with(backend: &dyn LspBackend, path: &str, src: &str) -> Result<(), SpineError> {
    let errors: Vec<String> = backend
        .diagnostics(path, src)
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| format!("line {}: {}", d.line, d.message))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(SpineError::Unverified(errors.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test-only backend: hand it the diagnostics you want it to report.
    /// Lets the verify tests assert behavior (clean vs. error) without
    /// hardcoding tsgo or SUI-LSP output.
    struct Scripted {
        diags: Vec<Diagnostic>,
    }
    impl LspBackend for Scripted {
        fn diagnostics(&self, _path: &str, _src: &str) -> Vec<Diagnostic> {
            self.diags.clone()
        }
        fn hover(&self, _path: &str, _src: &str, _line: u32, _col: u32) -> Option<String> {
            None
        }
    }

    // Routing: `.ts` -> TS, `.tsx` -> TS, `.sui` -> Sui. Assert on the kind
    // (the stable identity), not the concrete stub — the real wiring may swap
    // the inner type without changing the kind.
    #[test]
    fn backend_for_ts_routes_to_typescript() {
        assert_eq!(backend_for("foo.ts").kind(), BackendKind::TypeScript);
        assert_eq!(backend_for("foo.tsx").kind(), BackendKind::TypeScript);
    }

    #[test]
    fn backend_for_sui_routes_to_sui() {
        assert_eq!(backend_for("views/foo/bar.sui").kind(), BackendKind::Sui);
    }

    // Routing is total: every input, including unknown extensions and paths
    // with no extension at all, yields a defined `Backend`. We pin the
    // permissive default (TS) so a future narrowing of routing is a visible
    // decision, not a silent behavior change.
    #[test]
    fn backend_for_unknown_extension_routes_to_permissive_default() {
        assert_eq!(backend_for("README.md").kind(), BackendKind::TypeScript);
        assert_eq!(backend_for("no_extension").kind(), BackendKind::TypeScript);
        assert_eq!(backend_for("weird.xyz").kind(), BackendKind::TypeScript);
    }

    // Extension matching is case-insensitive on the final segment — files
    // from external tools routinely arrive as `.TS` or `.SUI`.
    #[test]
    fn backend_for_extension_match_is_case_insensitive() {
        assert_eq!(backend_for("Foo.TS").kind(), BackendKind::TypeScript);
        assert_eq!(backend_for("Foo.SUI").kind(), BackendKind::Sui);
    }

    // The public `verify_edit` is `verify_edit_with(backend_for(path), …)`:
    // a `.ts` path through a clean TS stub returns `Ok(())`.
    #[test]
    fn verify_edit_is_clean_when_ts_stub_reports_nothing() {
        let r = verify_edit("foo.ts", "const x: number = 1;");
        assert!(r.is_ok(), "clean TS stub must verify, got {r:?}");
    }

    // A `.sui` path through a clean SUI stub returns `Ok(())`.
    #[test]
    fn verify_edit_is_clean_when_sui_stub_reports_nothing() {
        let r = verify_edit("view.sui", "<view />");
        assert!(r.is_ok(), "clean SUI stub must verify, got {r:?}");
    }

    // The load-bearing behavior: when the routed backend reports an
    // `Error`-severity diagnostic, `verify_edit` refuses with
    // `SpineError::Unverified`. Tested via `verify_edit_with` so the
    // assertion is independent of which stub `backend_for` picks.
    #[test]
    fn verify_edit_refuses_on_error_severity() {
        let backend = Scripted {
            diags: vec![Diagnostic {
                severity: Severity::Error,
                message: "type mismatch".into(),
                line: 7,
            }],
        };
        let r = verify_edit_with(&backend, "foo.ts", "src");
        match r {
            Err(SpineError::Unverified(msg)) => {
                assert!(
                    msg.contains("type mismatch"),
                    "message must surface the diagnostic: {msg}"
                );
                assert!(msg.contains("7"), "message must surface the line: {msg}");
            }
            other => panic!("expected Unverified, got {other:?}"),
        }
    }

    // Warnings are NOT fatal — they pass the gate. The verify step is
    // adversarial against errors, not against style.
    #[test]
    fn verify_edit_accepts_warnings_only() {
        let backend = Scripted {
            diags: vec![Diagnostic {
                severity: Severity::Warning,
                message: "unused variable".into(),
                line: 3,
            }],
        };
        let r = verify_edit_with(&backend, "foo.ts", "src");
        assert!(r.is_ok(), "warnings must not refuse the edit, got {r:?}");
    }

    // Multiple errors are joined into one refusal message; the gate does not
    // silently drop the second error.
    #[test]
    fn verify_edit_collects_all_errors() {
        let backend = Scripted {
            diags: vec![
                Diagnostic {
                    severity: Severity::Error,
                    message: "first".into(),
                    line: 1,
                },
                Diagnostic {
                    severity: Severity::Error,
                    message: "second".into(),
                    line: 2,
                },
            ],
        };
        let r = verify_edit_with(&backend, "foo.ts", "src");
        match r {
            Err(SpineError::Unverified(msg)) => {
                assert!(msg.contains("first") && msg.contains("second"));
            }
            other => panic!("expected Unverified, got {other:?}"),
        }
    }

    // Mixed severities: warnings pass through, errors are the ones that
    // refuse. Order of evaluation must not matter.
    #[test]
    fn verify_edit_treats_only_errors_as_fatal() {
        let backend = Scripted {
            diags: vec![
                Diagnostic {
                    severity: Severity::Warning,
                    message: "w".into(),
                    line: 1,
                },
                Diagnostic {
                    severity: Severity::Error,
                    message: "e".into(),
                    line: 2,
                },
            ],
        };
        assert!(verify_edit_with(&backend, "foo.ts", "src").is_err());
    }

    // Optional registration: with an `lsp` grant, the SDK surface includes
    // `lsp.diagnostics` and `lsp.hover`, and the gateway treats both as
    // baseline (C0) — any agent, no grant required, may call them through
    // the one door.
    #[test]
    fn lsp_ops_are_registered_as_c0_baseline() {
        use crate::dtsgen::generate_dts;
        use crate::sdkgen::surface;
        let grants = vec!["lsp".to_string()];
        let surface_names: Vec<&str> = surface(&grants).iter().map(|i| i.export_name()).collect();
        assert!(
            surface_names.contains(&"lsp"),
            "lsp must appear in surface for an lsp grant; got {surface_names:?}"
        );

        let dts = generate_dts(&grants);
        assert!(
            dts.contains("diagnostics"),
            "lsp.diagnostics must be declared in the .d.ts"
        );
        assert!(
            dts.contains("hover"),
            "lsp.hover must be declared in the .d.ts"
        );
    }
}
