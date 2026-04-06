use std::path::PathBuf;

use runtime::set_active_agent;

pub fn set_agent(project_dir: PathBuf, id: Option<&str>) -> String {
    let ship_dir = project_dir.join(".ship");
    match set_active_agent(Some(ship_dir), id) {
        Ok(()) => match id {
            Some(id) => format!("Active agent set to '{}'", id),
            None => "Active agent cleared".to_string(),
        },
        Err(err) => format!("Error: {}", err),
    }
}
