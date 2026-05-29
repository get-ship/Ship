#!/usr/bin/env bash
# observe.sh — open a Windows Terminal tab with two split panes, each
# attached to one of the sui-eval fanout-lane tmux sessions.
#
# Usage:
#   bash .ship/pipelines/sui-eval/scripts/observe.sh           # latest run
#   bash .ship/pipelines/sui-eval/scripts/observe.sh <slug>    # specific run
#
# Why: the daemon spawns each lane in its OWN tmux session, so by
# default observing both means manually attaching two terminals.
# `ship run sui-eval` doesn't auto-open them (it dispatches via the
# daemon, not the operator's shell). This script discovers the latest
# lane-session pair from `tmux ls` and lays them out side-by-side in
# a single wt tab via `wt.exe new-tab` + `split-pane -V`.
#
# Requires: WSL2 + Windows Terminal (`wt.exe` on $PATH). For a
# tmux-native fallback (no wt.exe), see the "manual" block below.

set -euo pipefail

slug="${1:-}"

# If no slug given, pick the newest sui-eval run by scanning live
# tmux sessions for ANY lane suffix. Lanes can finish at different
# times (one calls step-done before the other), so anchoring on a
# specific suffix would miss in-flight runs where one lane has
# already exited.
if [ -z "$slug" ]; then
    slug=$(tmux ls -F '#{session_created} #{session_name}' 2>/dev/null \
        | awk '$2 ~ /^ship-sui-eval-[^-]+-build-[a-z]+$/ {print $1, $2}' \
        | sort -rn \
        | head -1 \
        | awk '{print $2}' \
        | sed -E 's/^ship-//; s/-build-[a-z]+$//')
fi

if [ -z "$slug" ]; then
    echo "observe-eval: no sui-eval-* tmux session found." >&2
    echo "  Did you 'ship run sui-eval' yet? Did the lanes spawn?" >&2
    exit 1
fi

HAIKU="ship-${slug}-build-haiku"
SONNET="ship-${slug}-build-sonnet"
HAIKU_ALIVE=true
SONNET_ALIVE=true
tmux has-session -t "$HAIKU" 2>/dev/null || HAIKU_ALIVE=false
tmux has-session -t "$SONNET" 2>/dev/null || SONNET_ALIVE=false

if ! $HAIKU_ALIVE && ! $SONNET_ALIVE; then
    echo "observe-eval: neither lane session is live for $slug." >&2
    exit 1
fi
if ! $HAIKU_ALIVE; then
    echo "observe-eval: haiku lane already exited; opening sonnet alone."
fi
if ! $SONNET_ALIVE; then
    echo "observe-eval: sonnet lane already exited; opening haiku alone."
fi

# wt.exe is the canonical path under WSL2. The Windows Terminal
# command-line spec uses `;` as the command separator; bash needs
# `\;` so the shell doesn't eat it. `new-tab` opens a tab with the
# haiku session, `split-pane -V` splits it vertically (side-by-side)
# and attaches the sonnet session.
if command -v wt.exe >/dev/null 2>&1; then
    if $HAIKU_ALIVE && $SONNET_ALIVE; then
        echo "observe-eval: opening wt tab for $slug (haiku | sonnet)"
        wt.exe new-tab \
            --title "sui-eval $slug" \
            wsl -e bash -c "tmux attach -t '$HAIKU'" \
            \; split-pane -V \
            wsl -e bash -c "tmux attach -t '$SONNET'"
    elif $HAIKU_ALIVE; then
        wt.exe new-tab --title "sui-eval $slug (haiku)" \
            wsl -e bash -c "tmux attach -t '$HAIKU'"
    else
        wt.exe new-tab --title "sui-eval $slug (sonnet)" \
            wsl -e bash -c "tmux attach -t '$SONNET'"
    fi
    exit 0
fi

# Fallback: build a single tmux session that joins both lane sessions
# as side-by-side panes. Works in any terminal without Windows
# Terminal — useful for operators on macOS / Linux / SSH'd-in WSL.
SESSION="sui-eval-observe-$slug"
tmux kill-session -t "$SESSION" 2>/dev/null || true
if $HAIKU_ALIVE && $SONNET_ALIVE; then
    tmux new-session -d -s "$SESSION" -n lanes "tmux attach -t '$HAIKU'"
    tmux split-window -h -t "$SESSION:lanes" "tmux attach -t '$SONNET'"
    tmux select-layout -t "$SESSION:lanes" even-horizontal
elif $HAIKU_ALIVE; then
    tmux new-session -d -s "$SESSION" -n lanes "tmux attach -t '$HAIKU'"
else
    tmux new-session -d -s "$SESSION" -n lanes "tmux attach -t '$SONNET'"
fi
echo "observe-eval: wt.exe not on PATH; created tmux session '$SESSION'."
echo "  attach with: tmux attach -t '$SESSION'"
