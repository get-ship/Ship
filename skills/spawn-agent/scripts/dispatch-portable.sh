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
# If helmd is unreachable at the default endpoint (no HELMD_URL / HELMD_ADDR
# override), this script tries to boot a local helmd from
# `<repo>/target/debug/helmd` (the dev build) or `helmd` on PATH, polls for
# readiness, and retries the POST once. Override HELMD_URL to point at a
# remote daemon when you don't want this fallback.
#
# Usage: bash dispatch-portable.sh --slug <name> --agent <agent> --spec <file> [options]
#   --harness <cli>   Harness to launch: claude (default) | omp | codex | ...
#   --model <id>      Model id (empty = harness default)
#   --base <branch>   Advisory only (back-compat). helmd always applies its
#                     own `job/<slug>` convention; the value is recorded in
#                     the dispatch log but NOT forwarded to helmd (sending
#                     it as `branch` would make helmd check out the
#                     integration branch instead of a fresh job branch).
#   --dir <path>      Worktree base dir (informational; helmd resolves its own)
#   --no-open         Accepted for back-compat (helmd manages the terminal)
#   --dry-run         Resolve + print the plan; do NOT POST helmd / spawn
# Env:
#   HELMD_URL         helmd base URL (default: http://${HELMD_ADDR:-127.0.0.1:2469})
#   HELMD_ADDR        host:port shorthand when HELMD_URL is unset
#   SHIP_WORKSPACE    optional workspace id forwarded to helmd
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
# Resolve worktree base via the shared helper sourced from this script's dir.
# Helpers: scripts/_worktree-base.sh — defines resolve_worktree_base()
_SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_worktree-base.sh
source "${_SELF_DIR}/_worktree-base.sh"
WORKTREE_BASE="$(resolve_worktree_base)"
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
  echo "  base:            $BASE_BRANCH  (advisory only — helmd applies job/<slug>)"
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
LIST_ENDPOINT="${HELMD_URL%/}/ship.ops.v1.AgentsService/List"

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
# The spec CONTENT is the dispatch PROMPT. helmd feeds `prompt` to the harness's
# `autostart_prompt`, and the tmux/pty adapter send-keys it once the TUI's ready
# pattern matches — so the agent starts with the job in hand, no `.artifacts`
# file required. (Previously `prompt` was hardcoded "" and the spec was passed as
# a PATH that staged an empty `.artifacts/job-spec.md`; the agent booted idle.)
# `spec` carries the same content so anything still reading `.artifacts/job-spec.md`
# gets the real body during the transition; `sourceSpecPath` keeps the origin
# path for re-dispatch. `--rawfile` JSON-escapes the multi-KB markdown safely.
if command -v jq >/dev/null 2>&1; then
  BODY="$(jq -n \
    --arg agent "$AGENT" --arg harness "$HARNESS" --arg model "$MODEL" \
    --arg srcpath "$SPEC_FILE" --arg slug "$SLUG" --arg workspace "$WORKSPACE" \
    --rawfile body "$SPEC_FILE" \
    '{agent:$agent, harness:$harness, model:$model, spec:$body,
      sourceSpecPath:$srcpath, slug:$slug, branch:"", workspace:$workspace,
      invocation:"pty", keepalive:false, prompt:$body}')"
else
  # No jq: build the JSON with python3 (safe escaping of the spec body).
  BODY="$(python3 - "$AGENT" "$HARNESS" "$MODEL" "$SPEC_FILE" "$SLUG" "$WORKSPACE" <<'PY'
import json, sys
agent, harness, model, spec_file, slug, workspace = sys.argv[1:7]
body = open(spec_file, encoding="utf-8").read()
print(json.dumps({
    "agent": agent, "harness": harness, "model": model,
    "spec": body, "sourceSpecPath": spec_file, "slug": slug,
    "branch": "", "workspace": workspace,
    "invocation": "pty", "keepalive": False, "prompt": body,
}))
PY
)"
fi

# POST and return "code\nbody" on stdout. The two-arg stdout-from-curl trick
# preserves the body even on 4xx/5xx (curl by default drops the body on error
# in --fail mode; we use the manual code-strip below to keep both).
post_dispatch() {
  # curl emits the body, then a trailing newline, then the HTTP code on its
  # own last line. We print it verbatim so callers can split with
  #   CODE="${RAW##*$'\n'}"; PAYLOAD="${RAW%$'\n'*}"
  # which is robust to bodies that contain embedded newlines.
  curl -sS -w $'\n%{http_code}' -X POST "$ENDPOINT" \
      -H "content-type: application/json" --data "$BODY" || {
    echo "helmd dispatch failed: could not reach $ENDPOINT" >&2
    return 1
  }
}

# Poll helmd for readiness on the ListAgents op. Returns 0 when a 2xx
# arrives (or ANY HTTP response — the daemon is up and answering), 1
# after the deadline elapses without any response.
wait_for_helmd() {
  local deadline=$((SECONDS + 10))
  while (( SECONDS < deadline )); do
    if curl -sS -o /dev/null -w '%{http_code}' --max-time 1 \
         -X POST "$LIST_ENDPOINT" \
         -H "content-type: application/json" -d '{}' 2>/dev/null \
         | grep -qE '^[0-9]{3}$'; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

# Try to start a local helmd. Looks for the dev build first, then PATH.
# Returns 0 if a candidate binary was launched, 1 if nothing was found.
boot_local_helmd() {
  local repo_root
  repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "")"
  local candidate=""
  if [[ -n "$repo_root" && -x "$repo_root/target/debug/helmd" ]]; then
    candidate="$repo_root/target/debug/helmd"
  elif command -v helmd >/dev/null 2>&1; then
    candidate="$(command -v helmd)"
  fi
  if [[ -z "$candidate" ]]; then
    echo "no helmd binary found (looked for <repo>/target/debug/helmd, then 'helmd' on PATH)" >&2
    return 1
  fi
  local log="${TMPDIR:-/tmp}/helmd-$$.log"
  nohup "$candidate" --addr "127.0.0.1:2469" >"$log" 2>&1 &
  disown || true
  echo "helmd was down; launched $candidate (logs: $log)" >&2
  return 0
}

# ── First attempt ─────────────────────────────────────────────────────────────
ATTEMPT=1
# First attempt. A non-zero exit here means curl couldn't reach the socket
# (DNS / connection refused / timeout). helmd returned a 4xx/5xx → curl
# exits 0 and we look at the code further down.
ATTEMPT=1
if ! RAW="$(post_dispatch 2>&1)"; then
  FIRST_ERR="$RAW"
  if [[ "$HELMD_URL" != "http://127.0.0.1:2469" ]]; then
    echo "helmd dispatch failed (operator-overridden HELMD_URL=$HELMD_URL, no local-boot fallback):" >&2
    echo "$FIRST_ERR" >&2
    exit 1
  fi
  if ! boot_local_helmd; then
    echo "first error: $FIRST_ERR" >&2
    exit 1
  fi
  if ! wait_for_helmd; then
    echo "helmd did not become ready within 10s; aborting" >&2
    exit 1
  fi
  echo "helmd is up; retrying dispatch" >&2
  ATTEMPT=2
  if ! RAW="$(post_dispatch 2>&1)"; then
    echo "helmd dispatch still failed after boot — check ${TMPDIR:-/tmp}/helmd-*.log" >&2
    echo "$RAW" >&2
    exit 1
  fi
fi
CODE="${RAW##*$'\n'}"
PAYLOAD="${RAW%$'\n'*}"

# Server returned a non-2xx. No retry — helmd rejected the request.
if [[ "$CODE" != 2* ]]; then
  echo "helmd dispatch error (HTTP $CODE): $PAYLOAD" >&2
  exit 1
fi

if [[ $ATTEMPT -gt 1 ]]; then
  echo "Dispatched via helmd ($ENDPOINT) on attempt $ATTEMPT:"
else
  echo "Dispatched via helmd ($ENDPOINT):"
fi
echo "  $PAYLOAD"
echo ""
echo "Launch manually:"
echo "  cd ${WORKTREE_PATH} && ${HARNESS}"
