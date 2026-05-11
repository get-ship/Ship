use std::path::PathBuf;

pub use ship_sui_host::resolver::{ResolvedView, ResolveError, render_resolved};

pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    if let Some(dir) = resolve_project_root() {
        ship_sui_host::resolver::resolve(&dir, name, sub_path)
    } else {
        let cwd = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => {
                return Err(ResolveError {
                    message: format!("view not found: {} — no cwd", join_request(name, sub_path)),
                });
            }
        };
        if let Some(ship_dir) = runtime::project::resolve_project_ship_dir_no_redirect(&cwd) {
            let project_root = ship_dir.parent().unwrap_or(&ship_dir).to_path_buf();
            ship_sui_host::resolver::resolve(&project_root, name, sub_path)
        } else {
            Err(ResolveError {
                message: format!(
                    "view not found: {} — no project root",
                    join_request(name, sub_path),
                ),
            })
        }
    }
}

fn resolve_project_root() -> Option<PathBuf> {
    let dir = std::env::var("SHIP_DEV_PROJECT_DIR").ok()?;
    Some(std::path::PathBuf::from(dir))
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}
