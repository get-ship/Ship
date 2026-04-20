#!/bin/sh
# RPM %post — runs as root after files are installed.
#
# cargo-generate-rpm threads this through to the spec's %post scriptlet.
# Semantics mirror the .deb postinst: drop to the invoking user and
# register the systemd user unit + autostart entries.
set -eu

TARGET_USER="${SUDO_USER:-}"
TARGET_UID="${SUDO_UID:-}"

if [ -z "$TARGET_USER" ] || [ -z "$TARGET_UID" ] || [ "$TARGET_USER" = "root" ]; then
    printf '[ship] no non-root target user; skipping autostart registration\n' >&2
    exit 0
fi

if command -v loginctl > /dev/null 2>&1; then
    loginctl enable-linger "$TARGET_USER" 2>/dev/null || true
fi

USER_HOME="$(getent passwd "$TARGET_USER" | cut -d: -f6)"
XDG_RUNTIME_DIR="/run/user/${TARGET_UID}"
export XDG_RUNTIME_DIR

su - "$TARGET_USER" -c "\
    XDG_RUNTIME_DIR='${XDG_RUNTIME_DIR}' \
    HOME='${USER_HOME}' \
    /usr/bin/ship daemon install-service --shipd-path /usr/bin/shipd \
" || true
