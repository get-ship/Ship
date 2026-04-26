//! `ship uninstall` — remove Ship's user-space state from this machine.
//!
//! Stops and unregisters the `shipd` user service (systemd --user on
//! Linux, launchd user agent on macOS), deletes `~/.ship/`, removes the
//! XDG desktop entry, and scrubs PATH entries added by the dev-fallback
//! `scripts/install.sh`. Confirms first unless `--yes` is set.
//!
//! **Does not remove the binaries themselves.** On a GUI install
//! (`.msi`, `.pkg`, `.deb`, `.rpm`) the platform uninstaller owns the
//! binary paths (`/usr/bin/ship`, `C:\Program Files\Ship\`, etc.) and
//! `dpkg --remove` / the MSI uninstaller / etc. is the right path to
//! clean them up. For the dev fallback the binaries live under
//! `~/.ship/bin/`, which goes away as part of removing `~/.ship/`.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sentinel comment written into rc files by `scripts/install.sh`. Used
/// here to find and strip the PATH block we added.
const PATH_SENTINEL: &str = "# ship-installer:path";

pub fn run(yes: bool) -> Result<()> {
    let home = dirs::home_dir().context("cannot resolve $HOME")?;
    let ship_home = runtime::env::read_raw(runtime::env::Var::Home)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| runtime::ShipHome::project_ship_dir(&home));

    if !yes && !confirm(&ship_home)? {
        println!("aborted.");
        return Ok(());
    }

    // Tear down the OS service registration via the canonical code
    // path — same module that `ship daemon uninstall-service` calls.
    // Failure is non-fatal so a partial install still cleans up.
    match crate::daemon::service_registration::uninstall_service() {
        Ok(report) => println!("{}", report.message),
        Err(e) => eprintln!("warning: service unregister failed: {e:#}"),
    }
    remove_desktop_entries(&home);
    remove_ship_home(&ship_home);
    scrub_path_entries(&home);

    println!();
    println!("ship has been uninstalled.");
    println!("  * open a new shell to drop the install dir from PATH");
    println!("  * reinstall with: curl -fsSL https://getship.dev/install.sh | sh");
    Ok(())
}

fn confirm(ship_home: &Path) -> Result<bool> {
    print!(
        "This will stop shipd, remove {}, and delete Ship's PATH entries from your shell rc files.\nContinue? [y/N] ",
        ship_home.display()
    );
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("failed to read confirmation")?;
    let answer = input.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn remove_desktop_entries(home: &Path) {
    // Linux XDG .desktop file written by some packaging layers.
    let desktop = home
        .join(".local")
        .join("share")
        .join("applications")
        .join("ship.desktop");
    if desktop.exists() {
        if let Err(e) = std::fs::remove_file(&desktop) {
            eprintln!("warning: could not remove {}: {e}", desktop.display());
        } else {
            println!("removed: {}", desktop.display());
        }
    }
}

fn remove_ship_home(ship_home: &Path) {
    if ship_home.exists() {
        if let Err(e) = std::fs::remove_dir_all(ship_home) {
            eprintln!("warning: could not remove {}: {e}", ship_home.display());
        } else {
            println!("removed: {}", ship_home.display());
        }
    }
}

fn scrub_path_entries(home: &Path) {
    let rcs = [
        home.join(".bashrc"),
        home.join(".zshrc"),
        home.join(".profile"),
        home.join(".config").join("fish").join("config.fish"),
    ];
    for rc in &rcs {
        if !rc.exists() {
            continue;
        }
        let Ok(contents) = std::fs::read_to_string(rc) else {
            continue;
        };
        if !contents.contains(PATH_SENTINEL) {
            continue;
        }
        let cleaned = strip_sentinel_block(&contents);
        if cleaned == contents {
            continue;
        }
        if let Err(e) = std::fs::write(rc, cleaned) {
            eprintln!("warning: could not rewrite {}: {e}", rc.display());
        } else {
            println!("cleaned: {}", rc.display());
        }
    }
}

/// Remove the `# ship-installer:path`-marked block and the single line
/// that immediately follows it. Leaves everything else untouched.
fn strip_sentinel_block(contents: &str) -> String {
    let mut out = Vec::new();
    let mut lines = contents.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == PATH_SENTINEL {
            // Drop the following line (the PATH export / fish_add_path).
            lines.next();
            continue;
        }
        out.push(line.to_string());
    }
    let trailing_newline = contents.ends_with('\n');
    let joined = out.join("\n");
    if trailing_newline {
        format!("{joined}\n")
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_sentinel_removes_block() {
        let input = "export FOO=1\n# ship-installer:path\nexport PATH=\"/x:$PATH\"\nexport BAR=2\n";
        let out = strip_sentinel_block(input);
        assert_eq!(out, "export FOO=1\nexport BAR=2\n");
    }

    #[test]
    fn strip_sentinel_noop_without_marker() {
        let input = "export FOO=1\nexport BAR=2\n";
        assert_eq!(strip_sentinel_block(input), input);
    }

    #[test]
    fn strip_sentinel_preserves_other_content() {
        let input = "# leading\n# ship-installer:path\nfish_add_path -g /x\n# trailing\n";
        let out = strip_sentinel_block(input);
        assert_eq!(out, "# leading\n# trailing\n");
    }
}
