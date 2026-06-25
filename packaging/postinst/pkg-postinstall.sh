#!/bin/sh
# macOS .pkg postinstall — runs as root after productbuild/pkgbuild drops
# files. cargo-dist (or a fallback productbuild invocation) references
# this script in the component package's Scripts/ payload.
#
# Jobs:
#   1. Copy the Ship.app launcher bundle into /Applications (cargo-dist
#      installs binaries under /usr/local/bin; the .app is packaged
#      alongside as a resource).
#   2. Register the launchd user agent for the target user by running
#      `ship daemon install-service` in their context.
set -eu

BIN_DIR="/usr/local/bin"
APP_SRC="${2:-/Applications/Ship.app}"
APP_DEST="/Applications/Ship.app"

if [ -d "$APP_SRC" ] && [ "$APP_SRC" != "$APP_DEST" ]; then
    /bin/rm -rf "$APP_DEST"
    /bin/cp -R "$APP_SRC" "$APP_DEST"
fi
/bin/chmod +x "$APP_DEST/Contents/MacOS/Ship" 2>/dev/null || true

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
    "${BIN_DIR}/ship" daemon install-service \
    --shipd-path "${BIN_DIR}/shipd" || {
        echo "[ship] ship daemon install-service failed; autostart not registered" >&2
        exit 0
    }

exit 0
