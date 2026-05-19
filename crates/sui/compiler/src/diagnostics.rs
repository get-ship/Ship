//! Span tracking and structured compile errors.
//!
//! Every IR node carries a `Span` pointing back at the user's source.
//! Errors capture the span, an optional file path, and a one-sentence
//! message so the CLI can render a useful error report.

use std::path::PathBuf;

/// A byte range into the original source. Inclusive start, exclusive end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn point(at: usize) -> Self {
        Self { start: at, end: at }
    }
}

/// Translate a byte offset to (line, column), both 1-based.
pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= clamped {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// A single compile-time error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct CompileError {
    pub message: String,
    pub span: Option<Span>,
    pub source_path: Option<PathBuf>,
    /// Optional category, e.g. `"<data>"` or the offending expression text.
    pub category: Option<String>,
}

impl CompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            source_path: None,
            category: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Human-readable rendering. Used by the CLI and snapshot tests.
    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();
        if let Some(p) = &self.source_path {
            out.push_str(&format!("{}: ", p.display()));
        }
        if let Some(s) = self.span {
            let (line, col) = line_col(source, s.start);
            out.push_str(&format!("line {line} col {col}: "));
        }
        if let Some(cat) = &self.category {
            out.push_str(&format!("[{cat}] "));
        }
        out.push_str(&self.message);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_newlines() {
        let src = "abc\ndef\nghi";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, 4), (2, 1));
        assert_eq!(line_col(src, 8), (3, 1));
    }

    #[test]
    fn line_col_clamps() {
        let src = "abc";
        assert_eq!(line_col(src, 99), (1, 4));
    }

    #[test]
    fn render_includes_path_and_position() {
        let err = CompileError::new("oops")
            .with_span(Span::new(4, 5))
            .with_path(PathBuf::from("foo.set"))
            .with_category("expr");
        let rendered = err.render("abc\ndef");
        assert!(rendered.contains("foo.set"));
        assert!(rendered.contains("line 2"));
        assert!(rendered.contains("[expr]"));
        assert!(rendered.contains("oops"));
    }
}
