use anyhow::Result;
use std::path::PathBuf;

pub(crate) const SHIPD_PORT: u16 = 9315;

pub(crate) fn ship_dir() -> Result<PathBuf> {
    Ok(ship_substrate::ShipHome::from_env().data_dir().to_path_buf())
}

pub(crate) fn pid_file() -> Result<PathBuf> {
    Ok(ship_substrate::home::current_daemon_dir().join("pid"))
}

pub(crate) fn log_file() -> Result<PathBuf> {
    Ok(ship_substrate::ShipHome::from_env()
        .state_dir()
        .join("logs")
        .join("shipd.log"))
}
