#!/bin/sh
# macOS .pkg postinstall — runs as root after productbuild/pkgbuild drops
# files. cargo-dist (or a fallback productbuild invocation) references
# this script in the component package's Scripts/ payload.
#
# Job: register the launchd user agent for the target user by running
# `shipctl daemon install-service` in their context. The desktop app is
# the Tauri shell, shipped by the desktop release — this .pkg carries
# the CLI + daemon only.
set -eu

BIN_DIR="/usr/local/bin"

# macOS .pkg installs to the machine; autostart / launchctl lives per
# user. Detect the "console user" — the one logged into the GUI — so
# the LaunchAgent lands in the right home directory.
CONSOLE_USER="$(/usr/bin/stat -f '%Su' /dev/console 2>/dev/null || echo root)"
CONSOLE_UID="$(/usr/bin/id -u "$CONSOLE_USER" 2>/dev/null || echo 0)"

if [ "$CONSOLE_USER" = "root" ] || [ "$CONSOLE_UID" = "0" ]; then
    echo "[ship] no console user detected; skipping LaunchAgent registration" >&2
    exit 0
fi

/usr/bin/sudo -u "$CONSOLE_USER" \
    "${BIN_DIR}/shipctl" daemon install-service \
    --shipd-path "${BIN_DIR}/shipd" || {
        echo "[ship] shipctl daemon install-service failed; autostart not registered" >&2
        exit 0
    }

exit 0
