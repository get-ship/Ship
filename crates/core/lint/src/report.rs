//! Finding types and output rendering.

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Security,
    Style,
    Signal,
    Pack,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
}
