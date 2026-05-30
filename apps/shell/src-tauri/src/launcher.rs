//! Per-app OS launcher generation — the core of the "every space is a real app"
//! model. There is one installed Ship runtime binary; each space gets its own
//! native launcher entry (icon + name) that invokes that shared binary with
//! `--app=<id>`. There is **no per-app build**: apps are SUI/data on one runtime,
//! so a launcher is OS metadata pointing at the existing executable, never a
//! second compiled program.
//!
//! Verification status (spike): Linux `.desktop` is implemented and tested
//! end-to-end on this host. macOS `.app` bundle generation is written to the
//! documented bundle layout but unverified on a non-mac host. Windows `.lnk`
//! generation is not written here — it needs a binary shell-link encoder
//! (`mslnk`/PowerShell) and a Windows host to verify, so it returns an explicit
//! error rather than a guess.

use std::path::{Path, PathBuf};

/// What an installed app launcher needs. `app_id` is passed to the runtime as
/// `--app`; the runtime binary itself is resolved from `current_exe()` so every
/// launcher points at the installed Ship.
#[derive(Debug, Clone)]
pub struct LauncherSpec {
    pub app_id: String,
    pub name: String,
    pub icon: PathBuf,
}

/// Render the freedesktop `.desktop` entry for `spec`, invoking `exe --app=<id>`.
/// Pure (no IO) so it is unit-testable on any host.
pub fn desktop_entry(spec: &LauncherSpec, exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Exec={exe} --app={id}\n\
         Icon={icon}\n\
         Terminal=false\n\
         Categories=Utility;\n\
         StartupNotify=true\n\
         X-Ship-App-Id={id}\n",
        name = spec.name,
        exe = exe.display(),
        id = spec.app_id,
        icon = spec.icon.display(),
    )
}

/// Render the macOS bundle `Info.plist` for `spec`. The bundle's executable is a
/// shell stub (`stub_name`) that execs the shared runtime with `--app`. Pure, so
/// it is unit-testable on any host.
pub fn macos_info_plist(spec: &LauncherSpec, stub_name: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>CFBundleName</key><string>{name}</string>\n\
         \t<key>CFBundleDisplayName</key><string>{name}</string>\n\
         \t<key>CFBundleIdentifier</key><string>com.micahcotton.ship.app.{id}</string>\n\
         \t<key>CFBundleExecutable</key><string>{stub}</string>\n\
         \t<key>CFBundleIconFile</key><string>icon.icns</string>\n\
         \t<key>CFBundlePackageType</key><string>APPL</string>\n\
         \t<key>CFBundleVersion</key><string>1.0</string>\n\
         </dict>\n\
         </plist>\n",
        name = spec.name,
        id = spec.app_id,
        stub = stub_name,
    )
}

/// Install a native launcher for `spec`, returning the path of the entry written.
/// Dispatches to the host OS implementation.
pub fn install(spec: &LauncherSpec) -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    install_for_host(spec, &exe)
}

#[cfg(target_os = "linux")]
fn install_for_host(spec: &LauncherSpec, exe: &Path) -> std::io::Result<PathBuf> {
    let dir = dirs_applications_linux()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("ship-{}.desktop", spec.app_id));
    std::fs::write(&path, desktop_entry(spec, exe))?;
    Ok(path)
}

/// `~/.local/share/applications` — the per-user freedesktop launcher dir.
#[cfg(target_os = "linux")]
fn dirs_applications_linux() -> std::io::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME unset"))?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("applications"))
}

#[cfg(target_os = "macos")]
fn install_for_host(spec: &LauncherSpec, exe: &Path) -> std::io::Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let home = std::env::var_os("HOME")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME unset"))?;
    let bundle = PathBuf::from(home)
        .join("Applications")
        .join(format!("{}.app", spec.name));
    let macos_dir = bundle.join("Contents").join("MacOS");
    let res_dir = bundle.join("Contents").join("Resources");
    std::fs::create_dir_all(&macos_dir)?;
    std::fs::create_dir_all(&res_dir)?;

    let stub_name = "ship-app";
    let stub = macos_dir.join(stub_name);
    std::fs::write(
        &stub,
        format!("#!/bin/sh\nexec \"{}\" --app={}\n", exe.display(), spec.app_id),
    )?;
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755))?;

    std::fs::write(
        bundle.join("Contents").join("Info.plist"),
        macos_info_plist(spec, stub_name),
    )?;
    if spec.icon.exists() {
        std::fs::copy(&spec.icon, res_dir.join("icon.icns"))?;
    }
    Ok(bundle)
}

#[cfg(target_os = "windows")]
fn install_for_host(_spec: &LauncherSpec, _exe: &Path) -> std::io::Result<PathBuf> {
    // A Windows launcher is a `.lnk` shell-link (binary format) plus an
    // AppUserModelID for distinct taskbar identity. Encoding `.lnk` correctly
    // needs a dedicated encoder (`mslnk`) or a PowerShell `WScript.Shell` call,
    // and a Windows host to verify. Not guessed here.
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows launcher generation not yet implemented (needs .lnk encoder + Windows host)",
    ))
}

/// Install a launcher from within Ship (the in-app convenience path the operator
/// asked for). Thin wrapper over `install`; the heavy native work lives in Rust.
#[tauri::command]
pub fn install_app_launcher(app_id: String, name: String, icon: String) -> Result<String, String> {
    let spec = LauncherSpec {
        app_id,
        name,
        icon: PathBuf::from(icon),
    };
    install(&spec)
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

/// Launch a space as its OWN process+window from the running shell: spawn the
/// same binary with `--app=<id>` (own PID → own webview, branded + routed for
/// the space; see `lib.rs::parse_app_id`). This is the process-per-space model's
/// core move — the hub opens spaces; each is an isolated process, so only the
/// hub window holds elevation. No per-app build: it's the same runtime + data.
#[tauri::command]
pub fn open_space(app_id: String) -> Result<u32, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let child = std::process::Command::new(exe)
        .arg(format!("--app={app_id}"))
        .spawn()
        .map_err(|e| format!("failed to spawn space '{app_id}': {e}"))?;
    Ok(child.id())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> LauncherSpec {
        LauncherSpec {
            app_id: "daily-digest".into(),
            name: "Daily Digest".into(),
            icon: PathBuf::from("/icons/digest.png"),
        }
    }

    #[test]
    fn desktop_entry_invokes_shared_exe_with_app_id() {
        let entry = desktop_entry(&spec(), Path::new("/opt/ship/ship-shell"));
        assert!(entry.contains("Exec=/opt/ship/ship-shell --app=daily-digest"));
        assert!(entry.contains("Name=Daily Digest"));
        assert!(entry.contains("Icon=/icons/digest.png"));
        assert!(entry.contains("X-Ship-App-Id=daily-digest"));
        assert!(entry.starts_with("[Desktop Entry]"));
    }

    #[test]
    fn info_plist_carries_name_id_and_stub() {
        let plist = macos_info_plist(&spec(), "ship-app");
        assert!(plist.contains("<string>Daily Digest</string>"));
        assert!(plist.contains("com.micahcotton.ship.app.daily-digest"));
        assert!(plist.contains("<key>CFBundleExecutable</key><string>ship-app</string>"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn install_writes_desktop_file_to_temp_home() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Scope HOME to the temp dir so the test never touches the real
        // applications dir.
        let prev = std::env::var_os("HOME");
        // SAFETY: single-threaded test; restored before return.
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let path = install(&spec()).unwrap();
        let restore = || match &prev {
            Some(v) => unsafe { std::env::set_var("HOME", v) },
            None => unsafe { std::env::remove_var("HOME") },
        };

        assert!(path.ends_with("ship-daily-digest.desktop"), "{path:?}");
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("--app=daily-digest"));
        restore();
    }
}
