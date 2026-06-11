//! Generate the daemon HTTP route reference (`cargo xtask gen-routes-doc`).
//!
//! SSOT: the `@route` doc annotations in `apps/shipd/src/routes/mod.rs` — the
//! same registry `apps/shipd/build.rs` compiles into the Rust router and the
//! TS route catalog, so the reference can never describe a route the daemon
//! doesn't register. Output is committed + drift-gated like the gen-skills
//! sdk pages (it lands beside them so the docs collector harvests it into the
//! site's `Ops:` reference unchanged):
//!
//!   * `skills/sdk/http-routes/SKILL.md` — agent-facing table of contents.
//!   * `skills/sdk/http-routes/references/docs/http-routes.mdoc` — the route
//!     table, stamped with `group: "Ops: http"` frontmatter so
//!     `apps/docs/scripts/collect-skill-docs.ts` renders it under the SDK
//!     (Ops) sidebar tree with zero astro.config changes.
//!
//! The output dir deliberately does NOT start with `ship-` — `gen-skills`
//! wipes every `ship-*` dir under `skills/sdk/` on regen and must not eat
//! this one.
//!
//! `--check` re-renders in-memory and byte-compares against the committed
//! files, exiting non-zero on drift (the CI gate). Each file is stamped
//! AUTO-GENERATED.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Route registry source, relative to the repo root.
const SRC_REL: &str = "apps/shipd/src/routes/mod.rs";

/// Output skill dir, relative to the repo root.
const OUT_DIR_REL: &str = "skills/sdk/http-routes";

#[derive(Debug, PartialEq)]
pub struct RouteDoc {
    pub variant: String,
    pub method: String,
    pub path: String,
    /// `Some("Full")` for `@auth scope Full`, `Some("handler")` for
    /// `@auth handler`, `None` when unannotated.
    pub auth: Option<String>,
    pub body: Option<String>,
    pub returns: Option<String>,
}

/// Parse every annotated variant out of the registry enum, in file order.
///
/// A `/// @route METHOD "path"` doc line opens a pending route; subsequent
/// `/// @returns` / `/// @body` / `/// @auth` lines enrich it; the first
/// following code line (skipping `//` comments, attributes, and blanks) is
/// the enum variant that closes it. The module doc's syntax example is
/// `//!`-prefixed and therefore never matches.
pub fn parse_routes(src: &str) -> Result<Vec<RouteDoc>> {
    let mut out = Vec::new();
    let mut pending: Option<(String, String)> = None; // (method, path)
    let mut auth: Option<String> = None;
    let mut body: Option<String> = None;
    let mut returns: Option<String> = None;

    for (lineno, line) in src.lines().enumerate() {
        let t = line.trim();
        if let Some(doc) = t.strip_prefix("///") {
            let doc = doc.trim();
            if let Some(rest) = doc.strip_prefix("@route ") {
                let (method, path) = parse_route_line(rest)
                    .with_context(|| format!("{SRC_REL}:{}: malformed @route", lineno + 1))?;
                pending = Some((method, path));
            } else if let Some(r) = doc.strip_prefix("@returns ") {
                returns = Some(r.trim().to_string());
            } else if let Some(b) = doc.strip_prefix("@body ") {
                body = Some(b.trim().to_string());
            } else if let Some(a) = doc.strip_prefix("@auth ") {
                let a = a.trim();
                auth = Some(match a.strip_prefix("scope ") {
                    Some(scope) => scope.trim().to_string(),
                    None => a.to_string(),
                });
            }
            continue;
        }
        // Non-doc line. Comments, attributes, and blanks never close a
        // pending route; the next real code line must be its variant.
        if t.is_empty() || t.starts_with("//") || t.starts_with('#') {
            continue;
        }
        if let Some((method, path)) = pending.take() {
            let variant: String = t
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if variant.is_empty() || !variant.starts_with(char::is_uppercase) {
                bail!(
                    "{SRC_REL}:{}: expected an enum variant after `@route {method} \"{path}\"`, got: {t}",
                    lineno + 1
                );
            }
            out.push(RouteDoc {
                variant,
                method,
                path,
                auth: auth.take(),
                body: body.take(),
                returns: returns.take(),
            });
        }
    }
    Ok(out)
}

/// `POST "/api/v1/session"` → `("POST", "/api/v1/session")`.
fn parse_route_line(rest: &str) -> Result<(String, String)> {
    let rest = rest.trim();
    let (method, quoted) = rest
        .split_once(' ')
        .with_context(|| format!("expected `METHOD \"path\"`, got: {rest}"))?;
    let path = quoted
        .trim()
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .with_context(|| format!("expected a quoted path, got: {quoted}"))?;
    Ok((method.to_string(), path.to_string()))
}

fn code_or_dash(v: Option<&str>) -> String {
    match v {
        Some(s) => format!("`{s}`"),
        None => "—".to_string(),
    }
}

fn render_skill_toc(routes: &[RouteDoc]) -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        "---\nname: http-routes\nstable-id: http-routes\naudience: public\ndescription: >-\n  The Ship daemon's plain-HTTP (REST) route surface ({n} routes) — method,\n  path, and required auth scope. This file is a table of contents; open\n  `references/docs/http-routes.mdoc` for the full reference table. The typed\n  Connect op surface is documented by the `ship-<domain>` sdk-skills next\n  door.\ntags: [sdk, api, http]\nauthors: [ship]\n---\n\n",
        n = routes.len(),
    );
    s.push_str(
        "<!-- AUTO-GENERATED by `cargo xtask gen-routes-doc` from apps/shipd/src/routes/mod.rs. Do not edit. -->\n\n# http-routes\n\nEvery HTTP route the daemon registers, parsed from the `@route` annotations\nin `apps/shipd/src/routes/mod.rs` — the registry `build.rs` compiles into the\nRust router and the TS route catalog.\n\n- [`http-routes`](references/docs/http-routes.mdoc) — the full method/path/auth table.\n",
    );
    s
}

fn render_page(routes: &[RouteDoc]) -> String {
    let mut s = String::new();
    s.push_str(
        "---\ntitle: HTTP routes\ndescription: Every HTTP route the Ship daemon registers — method, path, and required auth scope.\naudience: public\ngroup: \"Ops: http\"\norder: 0\n---\n\n",
    );
    s.push_str(
        "<!-- AUTO-GENERATED by `cargo xtask gen-routes-doc`. Source: apps/shipd/src/routes/mod.rs. Do not edit. -->\n\n# HTTP routes\n\n",
    );
    s.push_str(
        "The daemon's plain-HTTP (REST) surface: PTY sessions, artifacts, sandboxes,\nactor spawn, and daemon status. The typed op surface (rules, pipelines, git,\n…) is served over Connect instead — see the per-domain `Ops:` reference.\nParsed from the `@route` annotations in `apps/shipd/src/routes/mod.rs`, the\nsame registry `build.rs` compiles into the Rust router and the TS route\ncatalog, so this table cannot drift from what the daemon registers.\n\n",
    );
    s.push_str(
        "**Auth** — `scope <S>` routes are enforced against the named scope at the\nrouter. `handler` routes compute the required scope dynamically (e.g. from an\nartifact's stored scope) and enforce it inside the handler.\n\n",
    );
    s.push_str("| Method | Path | Auth | Body | Returns |\n|--------|------|------|------|---------|\n");
    for r in routes {
        let auth = match r.auth.as_deref() {
            Some("handler") => "handler".to_string(),
            Some(scope) => format!("scope `{scope}`"),
            None => "—".to_string(),
        };
        let _ = writeln!(
            s,
            "| `{}` | `{}` | {} | {} | {} |",
            r.method,
            r.path,
            auth,
            code_or_dash(r.body.as_deref()),
            code_or_dash(r.returns.as_deref()),
        );
    }
    s
}

pub fn run(root: &Path, check: bool) -> Result<()> {
    let src_path = root.join(SRC_REL);
    let src = std::fs::read_to_string(&src_path)
        .with_context(|| format!("reading {}", src_path.display()))?;
    let routes = parse_routes(&src)?;
    if routes.is_empty() {
        bail!("no @route annotations found in {SRC_REL} — registry moved?");
    }

    let out_dir = root.join(OUT_DIR_REL);
    let outputs = [
        (out_dir.join("SKILL.md"), render_skill_toc(&routes)),
        (out_dir.join("references/docs/http-routes.mdoc"), render_page(&routes)),
    ];

    if check {
        let mut stale = false;
        for (path, expected) in &outputs {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing != *expected {
                eprintln!("Out of date: {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-routes-doc` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("http-routes reference is up-to-date.");
        return Ok(());
    }

    for (path, content) in &outputs {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("mkdir {}", parent.display()))?;
        }
        std::fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
        eprintln!("gen-routes-doc: wrote {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNIPPET: &str = r#"
//! Registry doc. Syntax example (must never parse as a route):
//! /// @route METHOD "/path/with/{params}"

pub enum Route {
    /// @route POST "/api/v1/session"
    /// @returns SessionIssueResponse
    /// @auth scope Full
    IssueSession,

    // A plain comment between variants must not close a pending route.

    /// @route GET "/api/v1/artifacts/{id}"
    /// @auth handler
    #[allow(dead_code)]
    ArtifactDownload { id: String },

    /// @route POST "/api/v1/jobs"
    /// @body CreateJobRequest
    CreateJob,
}
"#;

    #[test]
    fn parses_method_path_auth_returns_and_variant() {
        let routes = parse_routes(SNIPPET).expect("snippet parses");
        assert_eq!(routes.len(), 3);
        let r = &routes[0];
        assert_eq!(r.variant, "IssueSession");
        assert_eq!(r.method, "POST");
        assert_eq!(r.path, "/api/v1/session");
        assert_eq!(r.auth.as_deref(), Some("Full"));
        assert_eq!(r.returns.as_deref(), Some("SessionIssueResponse"));
        assert_eq!(r.body, None);
    }

    #[test]
    fn handler_auth_braced_variant_and_attributes_are_handled() {
        let routes = parse_routes(SNIPPET).expect("snippet parses");
        let r = &routes[1];
        assert_eq!(r.variant, "ArtifactDownload");
        assert_eq!(r.auth.as_deref(), Some("handler"));
        // Optional annotations from the PREVIOUS route must not leak forward.
        assert_eq!(r.returns, None);
    }

    #[test]
    fn body_annotation_and_module_doc_example_ignored() {
        let routes = parse_routes(SNIPPET).expect("snippet parses");
        let r = &routes[2];
        assert_eq!(r.body.as_deref(), Some("CreateJobRequest"));
        assert_eq!(r.auth, None);
        // The `//!` syntax example contributed no route.
        assert!(routes.iter().all(|r| r.method != "METHOD"));
    }

    #[test]
    fn rendered_page_carries_frontmatter_group_and_rows() {
        let routes = parse_routes(SNIPPET).expect("snippet parses");
        let page = render_page(&routes);
        assert!(page.starts_with("---\ntitle: HTTP routes\n"));
        assert!(page.contains("group: \"Ops: http\""));
        assert!(page.contains("| `POST` | `/api/v1/session` | scope `Full` | — | `SessionIssueResponse` |"));
        assert!(page.contains("| `GET` | `/api/v1/artifacts/{id}` | handler | — | — |"));
    }

    #[test]
    fn malformed_variant_after_route_is_an_error() {
        let bad = "/// @route GET \"/x\"\n}\n";
        assert!(parse_routes(bad).is_err(), "a pending route closed by `}}` must fail loud");
    }
}
