---
group: Getting Started
title: Installation
section: guide
order: 4
audience: internal
---

# Installing Ship

> **Internal note:** Ship is in private beta. The public site shows a Private Beta badge linking to the waitlist instead of this page. Flip `audience` to `public` when public installers ship.

Three install paths, pick the one that matches how you live:

1. **GUI installer** — click a link on getship.dev, double-click the download. No terminal. For everyone.
2. **Package manager** — `brew`, `apt`, `dnf`, `winget`. The CLI path for people who already live in a terminal.
3. **Dev fallback** — `curl | sh` or `cargo install`. For contributors and exotic distros.

Every path installs the same two binaries (`ship` CLI + `shipd` daemon), registers `shipd` as a user-level service that autostarts at login, and opens the setup wizard in your browser on first run.

---

## macOS

### GUI

Download `ship-<version>-aarch64-apple-darwin.pkg` (Apple Silicon) or `...-x86_64-apple-darwin.pkg` (Intel) from the [releases page]. Double-click.

### CLI

```sh
# Homebrew (once the tap is live — see TODO)
brew install madvib/tap/ship

# Direct package install
curl -LO https://github.com/madvib/ship/releases/latest/download/ship-<version>-aarch64-apple-darwin.pkg
sudo installer -pkg ship-*.pkg -target /
```

### Uninstall

```sh
ship uninstall              # stops service, removes ~/.ship
sudo rm /usr/local/bin/ship /usr/local/bin/shipd
sudo rm -rf /Applications/Ship.app
```

---

## Linux (Debian, Ubuntu, Mint, Pop!_OS)

### CLI

```sh
curl -LO https://github.com/madvib/ship/releases/latest/download/ship_<version>_amd64.deb
sudo apt install ./ship_*.deb
```

The post-install hook registers `shipd` as a systemd `--user` unit and starts it in your session — no `systemctl` needed from you.

### GUI

Ubuntu's Discover / GNOME Software picks up `.deb` double-clicks; the flow is identical to the CLI path but with a wizard.

### Uninstall

```sh
sudo apt remove ship
ship uninstall              # removes ~/.ship + user-scoped state
```

---

## Linux (Fedora, RHEL, CentOS Stream, openSUSE)

### CLI

```sh
curl -LO https://github.com/madvib/ship/releases/latest/download/ship-<version>-1.x86_64.rpm
sudo dnf install ./ship-*.rpm
```

### Uninstall

```sh
sudo dnf remove ship
ship uninstall
```

---

## Windows

### GUI

Download `ship-<version>-x86_64-pc-windows-msvc.msi` from the [releases page]. Double-click. SmartScreen will warn until code signing lands — click **More info → Run anyway**.

### CLI

```powershell
# winget (when the manifest is published — TODO)
winget install Ship.Ship

# Direct MSI install (silent)
msiexec /i ship-<version>-x86_64-pc-windows-msvc.msi /quiet /norestart
```

### Uninstall

```powershell
msiexec /uninstall "Ship" /quiet
# Or: Settings → Apps → Installed apps → Ship → Uninstall
```

---

## Dev fallback (any Unix without native packages)

Used by Alpine, FreeBSD, airgapped boxes, and CI bootstraps:

```sh
curl -fsSL https://github.com/madvib/ship/releases/latest/download/install.sh | sh
```

This drops both binaries into `~/.ship/bin`, appends that dir to your shell's PATH, and calls `ship daemon install-service` to register systemd (Linux) or launchd (macOS). Idempotent on re-run.

Env knobs:

| Variable | Default | Effect |
|----------|---------|--------|
| `SHIP_VERSION` | `latest` | Pin a specific release, e.g. `0.2.0` |
| `SHIP_INSTALL_DIR` | `~/.ship/bin` | Where to drop binaries |
| `SHIP_SKIP_SERVICE` | `0` | Skip service registration |
| `SHIP_NO_BROWSER` | unset | Suppress first-run browser open |

## From source

```sh
git clone https://github.com/madvib/ship && cd ship
cargo install --path apps/ship-cli --path apps/shipd
ship daemon install-service       # register the service yourself
```

---

## TODOs (not blocking v1)

- **Homebrew tap** (`madvib/homebrew-tap`) — cargo-dist generates the Formula automatically once the tap repo exists and `HOMEBREW_TAP_TOKEN` is set on the release workflow. See `dist-workspace.toml` `tap` key.
- **winget manifest** — submit to microsoft/winget-pkgs after the first signed release; unsigned submissions are rejected.
- **Code signing** — see `contrib/installer-assets/SIGNING.md` for the SSL.com OV + Apple Developer Program rollout.

[releases page]: https://github.com/madvib/ship/releases/latest
