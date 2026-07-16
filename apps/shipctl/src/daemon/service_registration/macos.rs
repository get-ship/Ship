//! launchd user-agent registration for `shipd` on macOS.
//!
//! The plist lives in `~/Library/LaunchAgents/dev.getship.shipd.plist`
//! (per-user, no sudo required). `launchctl bootstrap gui/<uid>`
//! registers it for the current graphical session; we fall back to
//! `launchctl load` on older macOS where bootstrap isn't available.

use super::{InstallOptions, ServiceReport};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const LABEL: &str = "dev.getship.shipd";
const PLIST_FILENAME: &str = "dev.getship.shipd.plist";

pub fn install(shipd: &Path, opts: &InstallOptions) -> Result<ServiceReport> {
    let plist_path = user_plist_path()?;
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let ship_home = ship_home()?;
    let log_dir = ship_home.join("logs");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("mkdir {}", log_dir.display()))?;

    let contents = render_plist(shipd, &ship_home, &log_dir);
    std::fs::write(&plist_path, contents)
        .with_context(|| format!("write {}", plist_path.display()))?;

    // Idempotent (re)load. `bootout` fails silently when nothing is
    // registered yet — discard the error. `bootstrap` is the modern
    // launchctl entry point; fall back to `load` for older macOS.
    let _ = launchctl_bootout();
    if opts.enable_autostart {
        if launchctl_bootstrap(&plist_path).is_err() {
            launchctl_load(&plist_path)
                .context("launchctl load dev.getship.shipd.plist")?;
        }
    }
    if opts.start_now {
        // `kickstart -k` stops and immediately restarts the service,
        // which also applies new ExecStart paths after an upgrade.
        let _ = launchctl_kickstart();
    }

    Ok(ServiceReport::ok(format!(
        "shipd registered as a launchd user agent: {}",
        plist_path.display()
    )))
}

pub fn uninstall() -> Result<ServiceReport> {
    let plist_path = user_plist_path()?;
    let _ = launchctl_bootout();
    let _ = launchctl_unload(&plist_path);
    let removed = if plist_path.exists() {
        std::fs::remove_file(&plist_path)
            .with_context(|| format!("remove {}", plist_path.display()))?;
        true
    } else {
        false
    };
    Ok(ServiceReport::ok(if removed {
        format!("removed {}", plist_path.display())
    } else {
        "shipd launchd user agent was not registered".into()
    }))
}

fn user_plist_path() -> Result<PathBuf> {
    Ok(home::home_dir()
        .context("no $HOME")?
        .join("Library")
        .join("LaunchAgents")
        .join(PLIST_FILENAME))
}

fn ship_home() -> Result<PathBuf> {
    Ok(ship_substrate::ShipHome::from_env().data_dir().to_path_buf())
}

fn render_plist(shipd: &Path, ship_home: &Path, log_dir: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{shipd}</string>
        <string>--host</string>
        <string>127.0.0.1</string>
        <string>--port</string>
        <string>9315</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>5</integer>
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
    <key>StandardOutPath</key>
    <string>{log}/shipd.out.log</string>
    <key>StandardErrorPath</key>
    <string>{log}/shipd.err.log</string>
    <key>WorkingDirectory</key>
    <string>{home}</string>
    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>
"#,
        shipd = shipd.display(),
        log = log_dir.display(),
        home = ship_home.display()
    )
}

fn gui_domain() -> String {
    // launchctl's modern surface targets per-user "gui" domains.
    let uid = unsafe { libc::getuid() };
    format!("gui/{uid}")
}

fn launchctl_bootstrap(plist: &Path) -> Result<()> {
    let domain = gui_domain();
    let status = std::process::Command::new("launchctl")
        .args(["bootstrap", &domain])
        .arg(plist)
        .status()
        .context("exec launchctl bootstrap")?;
    anyhow::ensure!(status.success(), "launchctl bootstrap exited {status}");
    Ok(())
}

fn launchctl_bootout() -> Result<()> {
    let target = format!("{}/{LABEL}", gui_domain());
    let status = std::process::Command::new("launchctl")
        .args(["bootout", &target])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("exec launchctl bootout")?;
    anyhow::ensure!(status.success(), "launchctl bootout exited {status}");
    Ok(())
}

fn launchctl_load(plist: &Path) -> Result<()> {
    let status = std::process::Command::new("launchctl")
        .arg("load")
        .arg(plist)
        .status()
        .context("exec launchctl load")?;
    anyhow::ensure!(status.success(), "launchctl load exited {status}");
    Ok(())
}

fn launchctl_unload(plist: &Path) -> Result<()> {
    let status = std::process::Command::new("launchctl")
        .arg("unload")
        .arg(plist)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("exec launchctl unload")?;
    anyhow::ensure!(status.success(), "launchctl unload exited {status}");
    Ok(())
}

fn launchctl_kickstart() -> Result<()> {
    let target = format!("{}/{LABEL}", gui_domain());
    let status = std::process::Command::new("launchctl")
        .args(["kickstart", "-k", &target])
        .status()
        .context("exec launchctl kickstart")?;
    anyhow::ensure!(status.success(), "launchctl kickstart exited {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn render_plist_embeds_binary_path() {
        let plist = render_plist(
            &PathBuf::from("/usr/local/bin/shipd"),
            &PathBuf::from("/Users/alice/.ship"),
            &PathBuf::from("/Users/alice/.ship/logs"),
        );
        assert!(plist.contains("<string>/usr/local/bin/shipd</string>"));
        assert!(plist.contains("dev.getship.shipd"));
        assert!(plist.contains("/Users/alice/.ship/logs/shipd.out.log"));
    }

    #[test]
    fn render_plist_sets_keepalive() {
        let plist = render_plist(
            &PathBuf::from("/bin/shipd"),
            &PathBuf::from("/home/a"),
            &PathBuf::from("/home/a/logs"),
        );
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
    }
}
