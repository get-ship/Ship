//! Self-contained HTML error overlay for template render failures.
//!
//! When a user-authored Jinja template fails to parse or render, we produce
//! a styled error page that displays inline in the browser instead of
//! returning a blank 500 or swallowing the error silently.

use std::fmt::Write;

/// Try to extract a line number from a minijinja error message.
///
/// minijinja errors typically contain patterns like "line 7" or "at line 12".
fn extract_error_line(error: &str) -> Option<usize> {
    // Search for "line <number>" pattern.
    let lower = error.to_lowercase();
    let idx = lower.find("line ")?;
    let after = &error[idx + 5..];
    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

/// Escape HTML special characters for safe embedding.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render a self-contained HTML error overlay for a template failure.
///
/// The overlay uses Ship design token CSS variables (loaded by the shell)
/// with inline fallbacks so it looks reasonable even standalone.
pub fn render_error_overlay(view_name: &str, source: &str, error: &str) -> String {
    let error_line = extract_error_line(error);
    let source_block = build_source_block(source, error_line);
    let escaped_error = html_escape(error);
    let escaped_name = html_escape(view_name);

    let mut html = String::with_capacity(4096);
    // We use write! which is infallible on String, so unwrap is safe.
    write!(
        html,
        r##"<div class="ship-error-overlay" style="
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  padding: 1.5rem;
  max-width: 56rem;
  margin: 2rem auto;
  color: var(--foreground, #e4e4e7);
">
  <div style="
    background: var(--destructive, #7f1d1d);
    color: var(--destructive-foreground, #fef2f2);
    border-radius: 0.5rem;
    padding: 1rem 1.25rem;
    margin-bottom: 1.5rem;
  ">
    <div style="font-weight:700; font-size:1rem; margin-bottom:0.25rem;">
      Template Error in <code>{escaped_name}</code>
    </div>
    <pre style="
      margin:0; white-space:pre-wrap; word-break:break-word;
      font-family:'SF Mono',Menlo,Consolas,monospace; font-size:0.8125rem;
      line-height:1.5;
    ">{escaped_error}</pre>
  </div>
{source_block}
  <details style="margin-top:1.25rem;" open>
    <summary style="
      cursor:pointer; font-weight:600; font-size:0.875rem;
      color: var(--muted-foreground, #a1a1aa);
    ">Common mistakes</summary>
    <ul style="
      margin:0.5rem 0 0 1.25rem; padding:0;
      font-size:0.8125rem; line-height:1.7;
      color: var(--muted-foreground, #a1a1aa);
    ">
      <li>Missing <code>{{% endfor %}}</code> or <code>{{% endif %}}</code></li>
      <li>Undefined variable &mdash; use <code>| default('')</code> filter</li>
      <li>Typo in macro import: <code>{{% from '_ship.html.j2' import head %}}</code></li>
      <li>Wrong filter name or argument count</li>
      <li>Unbalanced quotes inside expressions</li>
    </ul>
  </details>
</div>"##
    )
    .unwrap();

    html
}

/// Build the source code block with line numbers and optional error highlighting.
fn build_source_block(source: &str, error_line: Option<usize>) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Determine which lines to show: 5 around the error, or first 20.
    let (start, end) = if let Some(eline) = error_line {
        let s = eline.saturating_sub(6); // 0-indexed: show 5 before
        let e = (eline + 4).min(lines.len()); // 5 after (1-indexed)
        (s, e)
    } else {
        (0, lines.len().min(20))
    };

    let gutter_width = format!("{}", end).len();
    let mut block = String::with_capacity(2048);
    write!(
        block,
        r#"  <div style="
    background: var(--card, #18181b);
    border: 1px solid var(--border, #27272a);
    border-radius: 0.5rem;
    overflow-x: auto;
  ">
    <div style="
      padding: 0.625rem 0;
      font-family:'SF Mono',Menlo,Consolas,monospace;
      font-size:0.75rem; line-height:1.6;
    ">"#
    )
    .unwrap();

    for (i, line_content) in lines[start..end].iter().enumerate() {
        let line_num = start + i + 1;
        let is_error = error_line == Some(line_num);
        let bg = if is_error {
            "background:var(--destructive,#7f1d1d);color:var(--destructive-foreground,#fef2f2);"
        } else {
            ""
        };
        let escaped = html_escape(line_content);
        write!(
            block,
            r#"
      <div style="display:flex;{bg}padding:0 1rem;">
        <span style="
          display:inline-block;width:{gutter_width}ch;
          text-align:right;margin-right:1rem;
          color:var(--muted-foreground,#52525b);user-select:none;
        ">{line_num}</span>
        <span style="flex:1;white-space:pre;">{escaped}</span>
      </div>"#
        )
        .unwrap();
    }

    if end < lines.len() {
        write!(
            block,
            r#"
      <div style="padding:0 1rem;color:var(--muted-foreground,#52525b);">
        ... ({} more lines)
      </div>"#,
            lines.len() - end
        )
        .unwrap();
    }

    write!(
        block,
        r#"
    </div>
  </div>"#
    )
    .unwrap();

    block
}

#[cfg(test)]
#[path = "error_overlay_tests.rs"]
mod tests;
