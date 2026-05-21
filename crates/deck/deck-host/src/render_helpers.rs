use std::path::Path;

pub fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

pub fn render_view_source(project_root: &Path, name: &str) -> Option<String> {
    let path = project_root.join("views").join(name).join("page.sui");
    std::fs::read_to_string(&path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_validation_rejects_traversal() {
        assert!(!is_valid_view_name(""));
        assert!(!is_valid_view_name("../etc"));
        assert!(!is_valid_view_name("Agents"));
        assert!(!is_valid_view_name("agents/index"));
        assert!(is_valid_view_name("agents"));
        assert!(is_valid_view_name("agent-detail"));
    }
}
