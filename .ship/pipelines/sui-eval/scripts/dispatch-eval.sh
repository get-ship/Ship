#!/usr/bin/env bash
# dispatch-eval.sh — fire a sui-eval run via dispatch.sh instead of the
# daemon's fanout path.
#
# Why: the daemon's fanout dispatch currently leaks badly (#193) and
# loses lane worktrees if the daemon is bounced mid-run. dispatch.sh
# spawns each agent as a one-off in a real PTY (interactive claude,
# NOT `-p --output-format stream-json`), so:
#   - lanes are fully observable (real TUI, not raw JSON)
#   - lanes survive daemon hiccups (no fanout coordinator)
#   - dispatch.sh opens a wt tab per lane automatically
#
# What we give up: the automatic step-done → grade → record chain.
# Score by hand after both lanes finish (or write a wrapper that
# polls for both worktrees to be quiet, then runs grade.ts +
# record.ts against each).
#
# Usage:
#   bash .ship/pipelines/sui-eval/scripts/dispatch-eval.sh
#   bash .ship/pipelines/sui-eval/scripts/dispatch-eval.sh --models haiku,sonnet,opus

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Default model set. Add/remove freely — each becomes one lane with
# its own dispatched worktree.
MODELS_CSV="haiku,sonnet"
SPEC=".ship/pipelines/sui-eval/task-spec.md"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --models) MODELS_CSV="$2"; shift 2 ;;
        --spec)   SPEC="$2"; shift 2 ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

if [[ ! -f "$SPEC" ]]; then
    echo "spec file missing: $SPEC" >&2
    exit 1
fi

# Map short labels to actual model IDs the dispatched agent profile honors.
model_id() {
    case "$1" in
        haiku)  echo "claude-haiku-4-5-20251001" ;;
        sonnet) echo "claude-sonnet-4-6" ;;
        opus)   echo "claude-opus-4-8" ;;
        *) echo "$1" ;;  # pass through anything custom
    esac
}

# Run id is a short timestamp + entropy — same shape as `ship run`
# pipeline_run_id but visibly hand-rolled (we're not going through the
# daemon's run.started). Lanes share this prefix so grade.ts can find
# the cohort.
RUN_ID="$(date -u +%y%m%d%H%M%S)-$(head -c 4 /dev/urandom | xxd -p)"
echo "dispatch-eval: RUN_ID=$RUN_ID"
echo "dispatch-eval: spec    =$SPEC"
echo "dispatch-eval: models  =$MODELS_CSV"
echo

IFS=',' read -ra MODELS <<< "$MODELS_CSV"
SLUGS=()
for label in "${MODELS[@]}"; do
    label="${label// /}"  # trim spaces
    [[ -z "$label" ]] && continue
    slug="eval-${label}-${RUN_ID}"
    SLUGS+=("$slug")
    model="$(model_id "$label")"
    echo "── dispatching $label ($model) → $slug"
    SHIP_AGENT_MODEL="$model" \
        bash "$REPO_ROOT/.ship/skills/spawn-agent/scripts/dispatch.sh" \
            --slug "$slug" \
            --agent ship-designer \
            --spec "$SPEC" \
            --no-open
    echo
done

# Print one combined block of attach + grade commands the operator can
# paste. dispatch.sh with --no-open just prints the launch line; we
# extend the tmux+wt incantation here so all lanes land in tabs of the
# SAME wt window.
echo "════════════════════════════════════════════════════════════════"
echo "RUN_ID:    $RUN_ID"
echo "lanes:     ${#SLUGS[@]}  (${SLUGS[*]})"
echo
echo "Launch the agents (one wt tab per lane in the focused window):"
for slug in "${SLUGS[@]}"; do
    worktree="$HOME/.ship/worktrees/$slug"
    cat <<EOF
  tmux new-session -d -s "$slug" -c "$worktree" 'claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship'
  wt.exe -w 0 new-tab --title "$slug" -- wsl.exe bash --login -c "tmux attach -t $slug"
EOF
done
echo
echo "Score (after both finish) — grade per worktree, then record:"
for slug in "${SLUGS[@]}"; do
    echo "  SHIP_PIPELINE_RUN_ID='$RUN_ID' bun .ship/pipelines/sui-eval/scripts/grade.ts # in $HOME/.ship/worktrees/$slug"
done
echo "  SHIP_PIPELINE_RUN_ID='$RUN_ID' bun .ship/pipelines/sui-eval/scripts/record.ts"
echo "════════════════════════════════════════════════════════════════"
