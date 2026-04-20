#!/bin/sh
# RPM %preun — runs as root before files are removed.
set -eu

# Only act on outright removal; skip the pre-removal that precedes
# upgrades ($1 == 1 means "upgrade, not remove").
if [ "${1:-0}" -gt 0 ]; then
    exit 0
fi

TARGET_USER="${SUDO_USER:-}"
TARGET_UID="${SUDO_UID:-}"

if [ -z "$TARGET_USER" ] || [ -z "$TARGET_UID" ] || [ "$TARGET_USER" = "root" ]; then
    exit 0
fi

USER_HOME="$(getent passwd "$TARGET_USER" | cut -d: -f6 2>/dev/null || echo '/home/'"$TARGET_USER")"
XDG_RUNTIME_DIR="/run/user/${TARGET_UID}"
export XDG_RUNTIME_DIR

su - "$TARGET_USER" -c "\
    XDG_RUNTIME_DIR='${XDG_RUNTIME_DIR}' \
    HOME='${USER_HOME}' \
    /usr/bin/ship daemon uninstall-service \
" >/dev/null 2>&1 || true
