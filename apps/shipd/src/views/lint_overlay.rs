//! Dev-mode lint findings overlay injected into rendered views.
//!
//! Only active when `SHIP_DEV=1` or `RUST_ENV=development` is set.
//! Zero inline `<script>` — clipboard copy uses Datastar `data-on:click`.

/// Returns true when the daemon is running in dev mode.
#[allow(dead_code)] // wired up when SHIP_DEV lint overlay feature is active
pub(super) fn is_dev_mode() -> bool {
    ship_substrate::env::read_raw(ship_substrate::env::Var::Dev).as_deref() == Ok("1")
        || std::env::var("RUST_ENV").as_deref() == Ok("development")
}

#[allow(dead_code)] // wired up when SHIP_DEV lint overlay feature is active
/// Inject a floating lint-findings panel at the end of the `<body>`.
///
/// Uses ship-badge for severity dots and a `<details>` for collapse/expand.
/// Click on a `file:line` cell copies the path to clipboard via Datastar.
pub(super) fn inject_lint_overlay(html: String, findings: &[lint::Finding]) -> String {
    let dot = |sev: lint::Severity| match sev {
        lint::Severity::Error => r#"variant="error""#,
        lint::Severity::Warn => r#"variant="warning""#,
    };

    let mut rows = String::new();
    for f in findings {
        let file = f.file.display().to_string();
        let loc = format!("{}:{}", f.line, f.col);
        let loc_full = format!("{}:{}", file, f.line);
        let badge_attr = dot(f.severity);
        let sev_label = f.severity.as_str();
        let rule = f.rule;
        let msg = &f.message;
        rows.push_str(&format!(
            "<tr style=\"vertical-align:top;line-height:1.4\">\
              <td style=\"padding:2px 6px 2px 0\">\
                <ship-badge {badge_attr} style=\"font-size:10px\">{sev_label}</ship-badge>\
              </td>\
              <td style=\"padding:2px 6px 2px 0;font-family:monospace;font-size:11px;\
                white-space:nowrap;cursor:pointer\"\
                data-on:click=\"navigator.clipboard.writeText('{loc_full}')\"\
                title=\"click to copy\">{loc}</td>\
              <td style=\"padding:2px 0;font-size:12px\">[{rule}] {msg}</td>\
            </tr>",
        ));
    }

    let errors = findings
        .iter()
        .filter(|f| f.severity == lint::Severity::Error)
        .count();
    let warns = findings
        .iter()
        .filter(|f| f.severity == lint::Severity::Warn)
        .count();
    let summary = format!("{errors} error(s), {warns} warning(s)");

    let panel = format!(
        r#"<details id="ship-lint-overlay" open style="position:fixed;bottom:16px;right:16px;z-index:9999;background:var(--ship-surface,#1e1e2e);border:1px solid var(--ship-border,#45475a);border-radius:8px;padding:10px 14px;box-shadow:0 4px 24px rgba(0,0,0,.5);max-width:600px;max-height:70vh;overflow:auto;font-size:13px;color:var(--ship-text,#cdd6f4)"><summary style="cursor:pointer;font-weight:600;list-style:none;margin-bottom:6px">Ship lint — {summary}</summary><table style="border-collapse:collapse;width:100%">{rows}</table></details>"#,
        summary = summary,
        rows = rows,
    );

    if let Some(pos) = html.rfind("</body>") {
        let mut out = html.clone();
        out.insert_str(pos, &panel);
        out
    } else {
        format!("{html}{panel}")
    }
}
