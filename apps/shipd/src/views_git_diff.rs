//! Unified-diff parser for the git detail view.
//!
//! Mirrors `apps/web/src/features/studio/session/diff-parser.ts` so the
//! daemon can emit pre-parsed structured markup for the
//! `<ship-git-diff>` component tree. Kept decoupled from the handler +
//! data loaders so it can be unit-tested in isolation and the other
//! files stay under the 300-line cap.

use serde::Serialize;

/// HTML-escape diff line content so that when Datastar morphs via innerHTML,
/// the decoded entities still render as text (not parsed as HTML elements).
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DiffLine {
    /// "add" | "del" | "context" — matches the component `type` attribute.
    pub kind: &'static str,
    pub content: String,
    pub old_num: Option<u32>,
    pub new_num: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct DiffFileView {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct ParsedDiff {
    pub files: Vec<DiffFileView>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

fn parse_hunk_header(line: &str) -> (u32, u32) {
    // Matches `@@ -<old>[,<n>] +<new>[,<n>] @@ ...`
    let rest = line.strip_prefix("@@").unwrap_or(line).trim_start();
    let mut old_start = 1u32;
    let mut new_start = 1u32;
    for tok in rest.split_whitespace() {
        if let Some(body) = tok.strip_prefix('-') {
            old_start = body
                .split(',')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
        } else if let Some(body) = tok.strip_prefix('+') {
            new_start = body
                .split(',')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            break;
        }
    }
    (old_start, new_start)
}

pub fn parse_unified_diff(text: &str) -> ParsedDiff {
    let mut files: Vec<DiffFileView> = Vec::new();
    let mut current_file: Option<DiffFileView> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line: u32 = 0;
    let mut new_line: u32 = 0;

    let flush_hunk = |file: &mut DiffFileView, hunk: &mut Option<DiffHunk>| {
        if let Some(h) = hunk.take() {
            file.hunks.push(h);
        }
    };

    for line in text.split('\n') {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            // push previous file
            if let Some(mut f) = current_file.take() {
                flush_hunk(&mut f, &mut current_hunk);
                files.push(f);
            }
            let path = rest
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.strip_prefix("b/"))
                .unwrap_or("unknown")
                .to_string();
            current_file = Some(DiffFileView {
                path,
                ..Default::default()
            });
            continue;
        }

        if line.starts_with("index ") || line.starts_with("--- ") {
            continue;
        }
        if let Some(p) = line.strip_prefix("+++ b/") {
            if let Some(f) = current_file.as_mut() {
                if f.path == "unknown" {
                    f.path = p.to_string();
                }
            }
            continue;
        }
        if line.starts_with("+++") {
            continue;
        }

        if line.starts_with("@@") {
            let Some(file) = current_file.as_mut() else {
                continue;
            };
            flush_hunk(file, &mut current_hunk);
            let (o, n) = parse_hunk_header(line);
            old_line = o;
            new_line = n;
            current_hunk = Some(DiffHunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
            continue;
        }

        let (Some(file), Some(hunk)) = (current_file.as_mut(), current_hunk.as_mut()) else {
            continue;
        };

        if let Some(rest) = line.strip_prefix('+') {
            if line.starts_with("+++") {
                continue;
            }
            hunk.lines.push(DiffLine {
                kind: "add",
                content: escape_html(rest),
                old_num: None,
                new_num: Some(new_line),
            });
            file.additions += 1;
            new_line += 1;
        } else if let Some(rest) = line.strip_prefix('-') {
            if line.starts_with("---") {
                continue;
            }
            hunk.lines.push(DiffLine {
                kind: "del",
                content: escape_html(rest),
                old_num: Some(old_line),
                new_num: None,
            });
            file.deletions += 1;
            old_line += 1;
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" — skip
            continue;
        } else {
            let content = escape_html(line.strip_prefix(' ').unwrap_or(line));
            hunk.lines.push(DiffLine {
                kind: "context",
                content,
                old_num: Some(old_line),
                new_num: Some(new_line),
            });
            old_line += 1;
            new_line += 1;
        }
    }

    if let Some(mut f) = current_file.take() {
        flush_hunk(&mut f, &mut current_hunk);
        files.push(f);
    }

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();
    ParsedDiff {
        files,
        total_additions,
        total_deletions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "diff --git a/src/foo.rs b/src/foo.rs\nindex abc..def 100644\n--- a/src/foo.rs\n+++ b/src/foo.rs\n@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"old\");\n+    println!(\"new\");\n+    let x = 1;\n }\n";

    #[test]
    fn parses_single_file() {
        let p = parse_unified_diff(SAMPLE);
        assert_eq!(p.files.len(), 1);
        let f = &p.files[0];
        assert_eq!(f.path, "src/foo.rs");
        assert_eq!(f.additions, 2);
        assert_eq!(f.deletions, 1);
        assert_eq!(f.hunks.len(), 1);
        assert_eq!(f.hunks[0].header, "@@ -1,3 +1,4 @@");
        // 5 "real" lines plus a trailing context produced by the final \n
        // — mirrors the JS parser's split('\n') behavior.
        assert_eq!(f.hunks[0].lines.len(), 6);
    }

    #[test]
    fn line_numbers_increment_correctly() {
        let p = parse_unified_diff(SAMPLE);
        let lines = &p.files[0].hunks[0].lines;
        assert_eq!(lines[0].kind, "context");
        assert_eq!(lines[0].old_num, Some(1));
        assert_eq!(lines[0].new_num, Some(1));
        assert_eq!(lines[1].kind, "del");
        assert_eq!(lines[1].old_num, Some(2));
        assert_eq!(lines[1].new_num, None);
        assert_eq!(lines[2].kind, "add");
        assert_eq!(lines[2].old_num, None);
        assert_eq!(lines[2].new_num, Some(2));
        assert_eq!(lines[3].kind, "add");
        assert_eq!(lines[3].new_num, Some(3));
    }

    #[test]
    fn totals_across_files() {
        let multi = format!("{SAMPLE}diff --git a/b.txt b/b.txt\n--- a/b.txt\n+++ b/b.txt\n@@ -1 +1 @@\n-a\n+b\n");
        let p = parse_unified_diff(&multi);
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.total_additions, 3);
        assert_eq!(p.total_deletions, 2);
    }

    #[test]
    fn empty_input_yields_no_files() {
        let p = parse_unified_diff("");
        assert!(p.files.is_empty());
        assert_eq!(p.total_additions, 0);
    }

    #[test]
    fn hunk_header_without_comma_counts() {
        let (o, n) = parse_hunk_header("@@ -5 +7 @@");
        assert_eq!(o, 5);
        assert_eq!(n, 7);
    }
}
