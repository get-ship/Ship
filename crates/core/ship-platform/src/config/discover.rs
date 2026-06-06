use ship_config::project::ProjectDiscovery;
use ship_substrate::project::SHIP_DIR_NAME;
use anyhow::Result;
use std::path::PathBuf;

pub fn discover_projects(root: PathBuf) -> Result<Vec<ProjectDiscovery>> {
    let mut projects = Vec::new();
    if !root.is_dir() {
        return Ok(projects);
    }

    // Check if root itself is a project (has .ship/ as a direct child).
    let root_ship = root.join(SHIP_DIR_NAME);
    if root_ship.exists() && root_ship.is_dir() {
        let name = root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        projects.push(ProjectDiscovery {
            name,
            path: root_ship,
        });
    }

    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip hidden, system, and archive directories
            if name.starts_with('.') && name != ".ship" {
                continue;
            }
            if matches!(
                name.as_ref(),
                "Trash"
                    | ".Trash"
                    | ".DS_Store"
                    | "._*"
                    | "TemporaryItems"
                    | ".Spotlight-V100"
                    | ".fseventsd"
            ) {
                continue;
            }
            let ship_dir = path.join(SHIP_DIR_NAME);
            if ship_dir.exists() && ship_dir.is_dir() {
                projects.push(ProjectDiscovery {
                    name: name.into_owned(),
                    path: ship_dir,
                });
            }
        }
    }
    Ok(projects)
}
