//! systemd --user registration for `shipd` on Linux.
//!
//! The unit file lives in `$XDG_CONFIG_HOME/systemd/user/shipd.service`
//! (default `~/.config/systemd/user/shipd.service`). `systemctl --user`
//! reads from there first and overrides any `/usr/lib/systemd/user/`
//! unit shipped by the distro package — so we always win.

use super::{InstallOptions, ServiceReport};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const UNIT_FILENAME: &str = "shipd.service";

pub fn install(shipd: &Path, opts: &InstallOptions) -> Result<ServiceReport> {
    let unit_path = user_unit_path()?;
    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let contents = render_unit(shipd);
    std::fs::write(&unit_path, contents)
        .with_context(|| format!("write {}", unit_path.display()))?;

    if !systemd_available() {
        return Ok(ServiceReport::ok(format!(
            "shipd unit written to {} (systemd user bus unavailable; \
             unit will load on next login)",
            unit_path.display()
        )));
    }

    run_systemctl(&["daemon-reload"]).ok();
    if opts.enable_autostart {
        run_systemctl(&["enable", UNIT_FILENAME])
            .context("systemctl --user enable shipd.service")?;
    }
    if opts.start_now {
        // `restart` doubles as "start if not running" and reloads
        // after a binary upgrade.
        run_systemctl(&["restart", UNIT_FILENAME])
            .context("systemctl --user restart shipd.service")?;
    }

    Ok(ServiceReport::ok(format!(
        "shipd registered as a systemd user service: {}",
        unit_path.display()
    )))
}

pub fn uninstall() -> Result<ServiceReport> {
    let unit_path = user_unit_path()?;
    if systemd_available() {
        let _ = run_systemctl(&["stop", UNIT_FILENAME]);
        let _ = run_systemctl(&["disable", UNIT_FILENAME]);
    }
    let removed = if unit_path.exists() {
        std::fs::remove_file(&unit_path)
            .with_context(|| format!("remove {}", unit_path.display()))?;
        true
    } else {
        false
    };
    if systemd_available() {
        let _ = run_systemctl(&["daemon-reload"]);
    }
    Ok(ServiceReport::ok(if removed {
        format!("removed {}", unit_path.display())
    } else {
        "shipd systemd user service was not registered".into()
    }))
}

fn user_unit_path() -> Result<PathBuf> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .context("no $HOME or $XDG_CONFIG_HOME")?;
    Ok(config_home.join("systemd").join("user").join(UNIT_FILENAME))
}

fn render_unit(shipd: &Path) -> String {
    format!(
        "\
[Unit]
Description=Ship daemon (shipd)
After=default.target
Documentation=https://getship.dev/docs

[Service]
Type=simple
ExecStart={shipd} --host 127.0.0.1 --port 9315
Restart=on-failure
RestartSec=2
TimeoutStopSec=30
Environment=\"RUST_LOG=info\"

[Install]
WantedBy=default.target
",
        shipd = shipd.display()
    )
}

fn systemd_available() -> bool {
    // `systemctl --user show-environment` probes the user DBus. Returns
    // non-zero in sealed containers / sshd-without-linger sessions — in
    // those cases we write the unit but skip enable/start.
    std::process::Command::new("systemctl")
        .args(["--user", "show-environment"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_systemctl(args: &[&str]) -> Result<()> {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("--user").args(args);
    let status = cmd
        .status()
        .with_context(|| format!("exec systemctl --user {}", args.join(" ")))?;
    anyhow::ensure!(
        status.success(),
        "systemctl --user {} exited {status}",
        args.join(" ")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn render_unit_inlines_shipd_path() {
        let unit = render_unit(&PathBuf::from("/usr/bin/shipd"));
        assert!(unit.contains("ExecStart=/usr/bin/shipd --host 127.0.0.1 --port 9315"));
        assert!(unit.contains("[Install]"));
        assert!(unit.contains("WantedBy=default.target"));
    }

    #[test]
    fn render_unit_contains_documentation_link() {
        let unit = render_unit(&PathBuf::from("/usr/bin/shipd"));
        assert!(unit.contains("Documentation=https://getship.dev/docs"));
    }
}
