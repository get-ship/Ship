#!/usr/bin/env bash
# lane-watch.sh — peer into each lane's tmux session and detect when the
# agent has gone idle, WITHOUT depending on the agent committing.
#
# Why: waiting on `git commit` is fragile — agents often finish the work but
# never commit, and a commit-based watcher then reads "no work done" and the
# cohort gets cleaned up. The tmux pane is ground truth: when Claude Code stops
# printing and its "esc to interrupt" working indicator is gone, the agent is
# done (or waiting on us). We watch the pane, not the branch.
#
# dispatch-eval.sh names each lane's tmux session after its slug
# (`eval-<model>-<RUN_ID>`), so we find the cohort by RUN_ID. When every lane is
# idle (or the global timeout hits), we grade + record automatically — one
# foreground command, no /loop, no commit race.
#
# Usage:
#   bash .ship/skills/sui-eval/scripts/lane-watch.sh <RUN_ID>
#   bash .ship/skills/sui-eval/scripts/lane-watch.sh <RUN_ID> --no-grade   # just signal

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
RUN_ID="${1:?usage: lane-watch.sh <RUN_ID> [--no-grade]}"
shift || true
DO_GRADE=1
[[ "${1:-}" == "--no-grade" ]] && DO_GRADE=0

WORKTREE_DIR="${SHIP_WORKTREE_DIR:-$HOME/.ship/worktrees}"
POLL_SECS=10          # how often to sample each pane
QUIET_POLLS=6         # consecutive unchanged + no-spinner samples → idle (60s)
TIMEOUT_SECS=2400     # global cap (40m) — never block forever
ACTIVE_MARKER='esc to interrupt'  # Claude Code's working indicator

# Find this run's lane sessions. dispatch-eval.sh sessions are `eval-*-<RUN_ID>`.
mapfile -t SESSIONS < <(tmux list-sessions -F '#{session_name}' 2>/dev/null \
  | grep -E "^eval-.*-${RUN_ID}$" || true)

if [[ ${#SESSIONS[@]} -eq 0 ]]; then
  echo "lane-watch: no tmux sessions match eval-*-${RUN_ID}" >&2
  echo "lane-watch: (lanes still booting? or RUN_ID wrong?)" >&2
  exit 1
fi

echo "lane-watch: RUN_ID=$RUN_ID watching ${#SESSIONS[@]} lane(s): ${SESSIONS[*]}"

declare -A STABLE PREV DONE
for s in "${SESSIONS[@]}"; do STABLE[$s]=0; PREV[$s]=""; DONE[$s]=0; done

SENTINEL_DIR="$REPO_ROOT/.artifacts/sui-eval-runs/.watch/$RUN_ID"
mkdir -p "$SENTINEL_DIR"

start=$SECONDS
while :; do
  all_done=1
  for s in "${SESSIONS[@]}"; do
    [[ ${DONE[$s]} -eq 1 ]] && continue

    # Session gone = agent process exited = done.
    if ! tmux has-session -t "$s" 2>/dev/null; then
      echo "lane-watch: $s — session ended (agent exited)"
      DONE[$s]=1; touch "$SENTINEL_DIR/$s.done"; continue
    fi

    pane="$(tmux capture-pane -t "$s" -p 2>/dev/null || true)"
    hash="$(printf '%s' "$pane" | cksum | cut -d' ' -f1)"
    active=0
    grep -qF "$ACTIVE_MARKER" <<<"$pane" && active=1

    if [[ $active -eq 1 || "$hash" != "${PREV[$s]}" ]]; then
      STABLE[$s]=0
    else
      STABLE[$s]=$(( STABLE[$s] + 1 ))
    fi
    PREV[$s]="$hash"

    if [[ ${STABLE[$s]} -ge $QUIET_POLLS ]]; then
      echo "lane-watch: $s — idle (${QUIET_POLLS}×${POLL_SECS}s quiet, no spinner)"
      DONE[$s]=1; touch "$SENTINEL_DIR/$s.done"
    else
      all_done=0
    fi
  done

  [[ $all_done -eq 1 ]] && break
  if (( SECONDS - start > TIMEOUT_SECS )); then
    echo "lane-watch: TIMEOUT after ${TIMEOUT_SECS}s — grading what's there" >&2
    break
  fi
  sleep "$POLL_SECS"
done

echo "lane-watch: all lanes settled."
if [[ $DO_GRADE -eq 0 ]]; then
  echo "lane-watch: --no-grade — sentinels in $SENTINEL_DIR"
  exit 0
fi

# Grade every lane worktree for this run, then record once.
echo "lane-watch: grading…"
for s in "${SESSIONS[@]}"; do
  wt="$WORKTREE_DIR/$s"
  if [[ -d "$wt" ]]; then
    ( cd "$wt" && SHIP_PIPELINE_RUN_ID="$RUN_ID" bun "$REPO_ROOT/.ship/skills/sui-eval/scripts/grade.ts" )
  else
    echo "lane-watch: $s — worktree missing at $wt" >&2
  fi
done
SHIP_PIPELINE_RUN_ID="$RUN_ID" bun "$REPO_ROOT/.ship/skills/sui-eval/scripts/record.ts"
echo "lane-watch: done — scores in views/eval + .artifacts/sui-eval-runs/$RUN_ID.jsonl"
