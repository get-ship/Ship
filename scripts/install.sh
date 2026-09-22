#!/usr/bin/env sh
# Dev fallback only — end users install via GUI installers from getship.dev/download.
#
# This script exists for developer / CI use: pulls the release tarball, drops
# the binaries under ~/.ship/bin, registers systemd-user or launchd, and opens
# the setup wizard. It is NOT part of the v1 distribution story. Keep it
# functional but do not invest in polish — .msi / .pkg / .deb / .rpm shipped
# from cargo-dist are the canonical installers.
#
# Usage (developers):
#   curl -fsSL https://github.com/madvib/ship/releases/download/vX.Y.Z/install.sh | sh
#
# Usage:
#   curl -fsSL https://getship.dev/install.sh | sh
#   curl -fsSL https://getship.dev/install.sh | sh -s -- --dry-run
#   curl -fsSL https://getship.dev/install.sh | sh -s -- --no-service
#   curl -fsSL https://getship.dev/install.sh | sh -s -- --no-browser
#
# Environment variables:
#   SHIP_VERSION       — pin a specific version (e.g. "0.1.0"). Default: latest.
#   SHIP_INSTALL_DIR   — installation directory. Default: ~/.ship/bin.
#   SHIP_HOME          — ship data directory. Default: ~/.ship.
#   SHIP_GITHUB_REPO   — GitHub repo. Default: madvib/ship.
#   SHIP_SKIP_SERVICE  — set to 1 to skip service registration.
#   SHIP_SKIP_BROWSER  — set to 1 to skip opening the setup wizard.
set -eu

REPO="${SHIP_GITHUB_REPO:-madvib/ship}"
SHIP_HOME="${SHIP_HOME:-$HOME/.ship}"
INSTALL_DIR="${SHIP_INSTALL_DIR:-$SHIP_HOME/bin}"
DEFAULT_PORT=9315
DRY_RUN=0
SKIP_SERVICE="${SHIP_SKIP_SERVICE:-0}"
SKIP_BROWSER="${SHIP_SKIP_BROWSER:-0}"

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        --no-service) SKIP_SERVICE=1 ;;
        --no-browser) SKIP_BROWSER=1 ;;
        *) printf "error: unknown argument: %s\n" "$arg" >&2; exit 1 ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────────────

say() { printf "  %s\n" "$*"; }
warn() { printf "  warning: %s\n" "$*" >&2; }
err() { printf "error: %s\n" "$*" >&2; exit 1; }

download() {
    url="$1"
    dest="$2"
    if command -v curl > /dev/null 2>&1; then
        if ! curl -fsSL "$url" -o "$dest" 2>/dev/null; then
            printf "error: download failed: %s\n" "$url" >&2
            printf "\n  Download manually: https://github.com/%s/releases/latest\n" "$REPO" >&2
            exit 1
        fi
    elif command -v wget > /dev/null 2>&1; then
        if ! wget -q "$url" -O "$dest" 2>/dev/null; then
            printf "error: download failed: %s\n" "$url" >&2
            printf "\n  Download manually: https://github.com/%s/releases/latest\n" "$REPO" >&2
            exit 1
        fi
    else
        err "neither curl nor wget found. Install one and retry:
    macOS:  brew install curl
    Debian: apt-get install curl
    Alpine: apk add curl"
    fi
}

# ── Detect platform ──────────────────────────────────────────────────────────

detect_platform() {
    _os="$(uname -s)"
    case "$_os" in
        Linux*)  _os="unknown-linux-gnu" ;;
        Darwin*) _os="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) _os="pc-windows-msvc" ;;
        *)
            err "unsupported OS: $_os
  Supported: Linux, macOS (Darwin)
  Open an issue: https://github.com/${REPO}/issues"
            ;;
    esac

    _arch="$(uname -m)"
    case "$_arch" in
        x86_64|amd64)  _arch="x86_64" ;;
        aarch64|arm64) _arch="aarch64" ;;
        *)
            err "unsupported architecture: $_arch
  Supported: x86_64, aarch64/arm64
  Open an issue: https://github.com/${REPO}/issues"
            ;;
    esac

    echo "${_arch}-${_os}"
}

detect_kernel() {
    case "$(uname -s)" in
        Linux*)  echo linux ;;
        Darwin*) echo darwin ;;
        MINGW*|MSYS*|CYGWIN*) echo windows ;;
        *) echo unknown ;;
    esac
}

# ── Resolve version ──────────────────────────────────────────────────────────

resolve_version() {
    if [ -n "${SHIP_VERSION:-}" ]; then
        echo "v${SHIP_VERSION#v}"
        return
    fi

    api_url="https://api.github.com/repos/${REPO}/releases/latest"
    tmpfile="$(mktemp)"
    download "$api_url" "$tmpfile"
    tag="$(grep '"tag_name"' "$tmpfile" | head -1 | sed 's/.*"tag_name": *"//;s/".*//')"
    rm -f "$tmpfile"

    if [ -z "$tag" ]; then
        err "could not determine latest version.
  Set SHIP_VERSION manually, e.g.:
    SHIP_VERSION=0.1.0 curl -fsSL https://getship.dev/install.sh | sh"
    fi
    echo "$tag"
}

# ── PATH management ──────────────────────────────────────────────────────────

# Append `export PATH="$INSTALL_DIR:$PATH"` to a shell rc file if not already
# present. Uses a sentinel comment so re-runs don't pile up duplicate entries.
path_append_posix() {
    rc="$1"
    [ -f "$rc" ] || return 0
    if grep -qs '# ship-installer:path' "$rc"; then
        return 0
    fi
    # Intentionally writing a literal "$PATH" into the rc file for the user's
    # shell to expand at source time, not now.
    # shellcheck disable=SC2016
    {
        printf '\n# ship-installer:path\nexport PATH="%s:$PATH"\n' "$INSTALL_DIR"
    } >> "$rc"
    say "PATH: appended to $rc"
}

path_append_fish() {
    rc="$HOME/.config/fish/config.fish"
    [ -f "$rc" ] || return 0
    if grep -qs '# ship-installer:path' "$rc"; then
        return 0
    fi
    mkdir -p "$(dirname "$rc")"
    {
        printf '\n# ship-installer:path\nfish_add_path -g %s\n' "$INSTALL_DIR"
    } >> "$rc"
    say "PATH: appended to $rc"
}

register_path() {
    # Create rc files if they don't exist yet — otherwise path_append_posix
    # silently skips. Matches the rc the user's default shell reads first.
    case "${SHELL:-}" in
        */bash) [ -f "$HOME/.bashrc" ] || : > "$HOME/.bashrc" ;;
        */zsh)  [ -f "$HOME/.zshrc"  ] || : > "$HOME/.zshrc"  ;;
    esac
    path_append_posix "$HOME/.bashrc"
    path_append_posix "$HOME/.zshrc"
    path_append_posix "$HOME/.profile"
    path_append_fish
}

# Service registration delegates to the `shipctl daemon install-service`
# subcommand. That command owns the systemd unit / launchd plist /
# Task Scheduler entry shapes — install.sh is not the place to keep a
# parallel copy of that logic.
register_service() {
    if [ "$SKIP_SERVICE" = "1" ]; then
        say "service: skipped (SHIP_SKIP_SERVICE=1)"
        return 0
    fi
    kernel="$(detect_kernel)"
    case "$kernel" in
        linux|darwin)
            if "${INSTALL_DIR}/shipctl" daemon install-service \
                    --shipd-path "${INSTALL_DIR}/shipd"; then
                say "service: registered via shipctl daemon install-service"
            else
                warn "shipctl daemon install-service returned non-zero — run it manually to enable autostart"
            fi
            ;;
        windows) warn "Windows service registration via install.sh is not supported — use the .msi installer" ;;
        *)       warn "unknown kernel; skipping service registration" ;;
    esac
}

# ── First-run browser ────────────────────────────────────────────────────────
#
# Browser opening is owned by the daemon itself (see
# `apps/shipd/src/first_run.rs`) — when shipd binds on a fresh install
# it spawns the platform's URL opener automatically. This function is
# kept as a no-op stub so existing install.test.sh smoke checks that
# reference `open_setup_if_first_run` still find the symbol.

open_setup_if_first_run() {
    # No-op: the daemon opens the browser itself on first bind
    # (see apps/shipd/src/first_run.rs).
    [ "$SKIP_BROWSER" = "1" ] || true
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
    printf "\n  Ship installer\n\n"

    platform="$(detect_platform)"
    version="$(resolve_version)"
    version_raw="${version#v}"
    say "platform:  $platform"
    say "version:   $version"
    say "install:   $INSTALL_DIR"

    ext="tar.gz"
    case "$platform" in
        *windows*) ext="zip" ;;
    esac

    archive="ship-${version_raw}-${platform}.${ext}"
    url="https://github.com/${REPO}/releases/download/${version}/${archive}"
    say "bundle:    $url"
    say ""

    if [ "$DRY_RUN" = "1" ]; then
        say "(dry run — nothing downloaded or installed)"
        return 0
    fi

    mkdir -p "$INSTALL_DIR" "$SHIP_HOME"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    say "downloading..."
    download "$url" "${tmpdir}/archive.${ext}"

    say "extracting..."
    case "$ext" in
        tar.gz)
            tar -xzf "${tmpdir}/archive.tar.gz" -C "$tmpdir"
            ;;
        zip)
            if ! command -v unzip > /dev/null 2>&1; then
                err "unzip not found. Install it and retry:
    Debian: apt-get install unzip"
            fi
            unzip -q "${tmpdir}/archive.zip" -d "$tmpdir"
            ;;
    esac

    # The archive extracts to a `ship-<ver>-<target>/` staging directory.
    # Fall back to finding the binaries directly in case the layout changes.
    shipctl_bin="$(find "$tmpdir" -name 'shipctl' -type f -perm -u+x 2>/dev/null | head -1)"
    [ -z "$shipctl_bin" ] && shipctl_bin="$(find "$tmpdir" -name 'shipctl' -type f 2>/dev/null | head -1)"
    [ -z "$shipctl_bin" ] && shipctl_bin="$(find "$tmpdir" -name 'shipctl.exe' -type f 2>/dev/null | head -1)"
    shipd_bin="$(find "$tmpdir" -name 'shipd' -type f -perm -u+x 2>/dev/null | head -1)"
    [ -z "$shipd_bin" ] && shipd_bin="$(find "$tmpdir" -name 'shipd' -type f 2>/dev/null | head -1)"
    [ -z "$shipd_bin" ] && shipd_bin="$(find "$tmpdir" -name 'shipd.exe' -type f 2>/dev/null | head -1)"

    if [ -z "$shipctl_bin" ] || [ -z "$shipd_bin" ]; then
        err "archive is missing shipctl and/or shipd. Report at: https://github.com/${REPO}/issues"
    fi

    install_file() {
        src="$1"
        dest="$2"
        # Atomic replace: copy to a sidecar then rename, so a concurrent
        # `shipd` process keeps executing the old inode until restart.
        cp "$src" "${dest}.new"
        chmod +x "${dest}.new"
        mv -f "${dest}.new" "$dest"
    }
    install_file "$shipctl_bin" "${INSTALL_DIR}/shipctl"
    install_file "$shipd_bin" "${INSTALL_DIR}/shipd"
    say "installed: ${INSTALL_DIR}/shipctl"
    say "installed: ${INSTALL_DIR}/shipd"

    register_path
    register_service
    open_setup_if_first_run

    say ""
    say "done. Next:"
    say "  * open a new shell so PATH picks up $INSTALL_DIR"
    say "  * visit http://localhost:${DEFAULT_PORT}/setup in your browser"
    say ""
}

main "$@"
