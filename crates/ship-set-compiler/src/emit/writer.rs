//! Indent-aware string writer. Keeps the JS emitter readable.

#[derive(Debug, Default)]
pub struct Writer {
    buf: String,
    indent: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_str(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn nl(&mut self) {
        self.buf.push('\n');
    }

    pub fn indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }

    pub fn line(&mut self, s: &str) {
        self.indent();
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    pub fn enter(&mut self) {
        self.indent += 1;
    }

    pub fn leave(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn into_string(self) -> String {
        self.buf
    }
}

/// Quote a string as a JS double-quoted literal.
pub fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
