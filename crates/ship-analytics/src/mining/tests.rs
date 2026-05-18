//! Mining tests. Today only smoke-test the stub returns sensible
//! defaults; real format-parsing tests land alongside the parser
//! implementations.

#[cfg(test)]
mod tests {
    use super::super::claude;
    use std::path::PathBuf;

    #[test]
    fn claude_scan_missing_dir_returns_empty() {
        let nonexistent = PathBuf::from("/tmp/ship-analytics-test-nonexistent");
        let rows = claude::scan(&nonexistent).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn claude_default_data_dir_under_home() {
        let p = claude::default_data_dir();
        // HOME is set in normal dev environments; if it isn't, the
        // helper returns None which the caller handles.
        if let Some(p) = p {
            assert!(p.to_string_lossy().contains(".claude/projects"));
        }
    }
}
