#!/usr/bin/env bash
# Ship dispatch (portable) — a THIN helmd client.
#
# This script no longer compiles agents, creates worktrees, or launches
# harnesses. helmd owns ALL of that now (ship.ops.v1.AgentsService/Dispatch
# compiles in-dispatch, provisions the worktree, and launches the harness).
# This wrapper just translates its CLI into a Connect-JSON POST to helmd —
# every consumer (the helm SPA, the MCP dispatch_job tool via helm's dispatch
# lib, an operator at the shell) is an RPC CLIENT of the ONE dispatch path.
#
# Usage: bash dispatch-portable.sh --slug <name> --agent <agent> --spec <file> [options]
#   --harness <cli>   Harness to launch: claude (default) | omp | codex | ...
#   --model <id>      Model id (empty = harness default)
#   --base <branch>   Branch to fork from (default: current branch)
#   --dir <path>      Worktree base dir (informational; helmd resolves its own)
#   --no-open         Accepted for back-compat (helmd manages the terminal)
#   --dry-run         Resolve + print the plan; do NOT POST helmd / spawn
# Env:
#   HELMD_URL         helmd base URL (default: http://${HELMD_ADDR:-127.0.0.1:2469})
#   HELMD_ADDR        host:port shorthand when HELMD_URL is unset
#   SHIP_WORKSPACE    optional workspace id forwarded to helmd
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
WORKTREE_BASE="${SHIP_WORKTREE_DIR:-${HOME}/dev/ship-worktrees}"
WORKTREE_BASE="${WORKTREE_BASE/#\~/$HOME}"
BASE_BRANCH=""
DRY_RUN=false
SLUG=""
AGENT=""
SPEC_FILE=""
HARNESS="${SHIP_PROVIDER_CLI:-}"
MODEL="${SHIP_AGENT_MODEL:-}"
WORKSPACE="${SHIP_WORKSPACE:-}"

# ── Parse args ────────────────────────────────────────────────────────────────
# Preserve the historical CLI. Flags that only mattered to the old in-script
# spawn (--skill/--confirm/--batch/--no-bootstrap/--ship-bootstrap/--no-open)
# are accepted as no-ops so existing callers don't break.
while [[ $# -gt 0 ]]; do
  case "$1" in
    --slug)     SLUG="$2"; shift 2 ;;
    --agent)    AGENT="$2"; shift 2 ;;
    --spec)     SPEC_FILE="$2"; shift 2 ;;
    --base)     BASE_BRANCH="$2"; shift 2 ;;
    --dir)      WORKTREE_BASE="$2"; shift 2 ;;
    --model)    MODEL="$2"; shift 2 ;;
    --harness)  HARNESS="$2"; shift 2 ;;
    --dry-run)  DRY_RUN=true; shift ;;
    --no-open)  shift ;;
    --skill)    shift 2 ;;
    --confirm)  shift ;;
    --batch)    shift 2 ;;
    --no-bootstrap)   shift ;;
    --ship-bootstrap) shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$SLUG" || -z "$AGENT" || -z "$SPEC_FILE" ]]; then
  echo "Usage: dispatch-portable.sh --slug <name> --agent <agent> --spec <file> [--harness <cli>] [--model <id>] [--base <branch>] [--dir <path>] [--dry-run]" >&2
  exit 1
fi
if [[ ! -f "$SPEC_FILE" ]]; then
  echo "Spec file not found: $SPEC_FILE" >&2
  exit 1
fi

HARNESS="${HARNESS:-claude}"
WORKTREE_PATH="${WORKTREE_BASE}/${SLUG}"
BRANCH_NAME="job/${SLUG}"
BASE_BRANCH="${BASE_BRANCH:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")}"

# ── Dry run ───────────────────────────────────────────────────────────────────
# The plan block (worktree/branch/base) is parsed by helm's dispatch lib
# (src/dispatch.ts) — keep the `  <label>:<spaces><value>` shape stable.
if $DRY_RUN; then
  echo "dispatch-portable (dry run — no helmd call):"
  echo "  slug:            $SLUG"
  echo "  agent:           $AGENT"
  echo "  harness:         $HARNESS"
  echo "  spec:            $SPEC_FILE"
  echo "  base:            $BASE_BRANCH"
  echo "  worktree:        $WORKTREE_PATH"
  echo "  branch:          $BRANCH_NAME"
  echo ""
  echo "Launch manually:"
  echo "  cd ${WORKTREE_PATH} && ${HARNESS}"
  exit 0
fi

# ── POST helmd.AgentsService/Dispatch ──────────────────────────────────────────
HELMD_URL="${HELMD_URL:-http://${HELMD_ADDR:-127.0.0.1:2469}}"
ENDPOINT="${HELMD_URL%/}/ship.ops.v1.AgentsService/Dispatch"

# Build the Connect-JSON request body (camelCase per the proto JSON mapping).
# Prefer jq for correct escaping; fall back to a heredoc when jq is absent.
# `branch` is helmd's JOB-branch name (it provisions the worktree on it), NOT a
# base-to-fork-from; DispatchAgentRequest has no base param. Leave it empty so
# helmd applies its own `job/<slug>` convention. --base is accepted for CLI
# back-compat but is advisory only (not forwarded — sending it as `branch` would
# make helmd check out the integration branch instead of a fresh job branch).
#
# keepalive:false → this CLI dispatch routes to the TmuxWindow executor: an
# interactive agent in a tmux tab you can `tmux attach -t ship-<slug>` to.
# keepalive:true would force the ProcessExecutor (the WS-attach / helm-UI
# surface) and no terminal tab would open. The SPA picks its own surface.
if command -v jq >/dev/null 2>&1; then
  BODY="$(jq -n \
    --arg agent "$AGENT" --arg harness "$HARNESS" --arg model "$MODEL" \
    --arg spec "$SPEC_FILE" --arg slug "$SLUG" --arg workspace "$WORKSPACE" \
    '{agent:$agent, harness:$harness, model:$model, spec:$spec,
      sourceSpecPath:$spec, slug:$slug, branch:"", workspace:$workspace,
      invocation:"pty", keepalive:false, prompt:""}')"
else
  BODY="$(cat <<JSON
{"agent":"$AGENT","harness":"$HARNESS","model":"$MODEL","spec":"$SPEC_FILE","sourceSpecPath":"$SPEC_FILE","slug":"$SLUG","branch":"","workspace":"$WORKSPACE","invocation":"pty","keepalive":false,"prompt":""}
JSON
)"
fi

RESP="$(curl -sS -w $'\n%{http_code}' -X POST "$ENDPOINT" \
  -H "content-type: application/json" -d "$BODY")" || {
  echo "helmd dispatch failed: could not reach $ENDPOINT" >&2
  exit 1
}
CODE="${RESP##*$'\n'}"
PAYLOAD="${RESP%$'\n'*}"

if [[ "$CODE" != 2* ]]; then
  echo "helmd dispatch error (HTTP $CODE): $PAYLOAD" >&2
  exit 1
fi

echo "Dispatched via helmd ($ENDPOINT):"
echo "  $PAYLOAD"
echo ""
echo "Launch manually:"
echo "  cd ${WORKTREE_PATH} && ${HARNESS}"
