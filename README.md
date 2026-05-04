# Ship

**Ship is an application runtime for AI-native applications.** Describe what your app should do; Ship runs it. Local-first, with cloud sync as an optional paid layer.

The 80% path: open Studio, describe your app in plain language, a first-party agent scaffolds the components — collections, agents, pipelines, views, permissions — and the app runs in five minutes. Edit any of it; the runtime hot-reloads. Hand the project to a friend; they install and run the same app.

The 20% path: hand-author components directly. Same primitives, same runtime.

[`docs/what-is-ship.md`](docs/what-is-ship.md) has the full description — primitives, taxonomy, storage modalities, what's in v1.

## Get started

1. Download Ship from the [releases page] and run the installer (`.pkg` / `.msi` / `.deb` / `.rpm`).
2. Studio launches in your browser at `http://localhost:9315`.
3. Describe what you want to build.

[releases page]: https://github.com/madvib/ship/releases/latest

## Install

Full per-platform matrix in [`docs/INSTALL.md`](docs/INSTALL.md). Short form:

```sh
# Debian/Ubuntu  sudo apt install ./ship_*.deb
# Fedora/RHEL    sudo dnf install ./ship-*.rpm
# Windows        msiexec /i ship-*.msi

# Dev / exotic Unix — bootstraps both binaries into ~/.ship/bin:
curl -fsSL https://github.com/madvib/ship/releases/latest/download/install.sh | sh

# From source:
cargo install --path apps/ship-cli --path apps/shipd
```

## Status

Active development; v1 in flight. The migration plan is at [`docs/migration-plan.md`](docs/migration-plan.md).

Breaking changes on the runtime, Studio, and rendering surfaces should be expected until v1.
