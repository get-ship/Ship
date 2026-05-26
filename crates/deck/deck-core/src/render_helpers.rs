pub use deck_host::render_helpers::is_valid_view_name;

pub fn render_view_source(name: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = ship_substrate::project::resolve_project_ship_dir(&cwd)?;
    let project_root = ship_dir.parent()?;
    deck_host::render_helpers::render_view_source(project_root, name)
}
