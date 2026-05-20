#!/usr/bin/env bash
# Ship dispatch — create a worktree, write a job spec, compile the agent, open a terminal.
# Usage: bash scripts/dispatch.sh --slug <name> --agent <agent> --spec <file> [options]
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
SHIP_GLOBAL_DIR="${SHIP_GLOBAL_DIR:-$HOME/.ship}"

# Resolve worktree base. Precedence: --dir flag > SHIP_WORKTREE_DIR env >
# `ship config get worktrees.dir` > legacy HOME default.
resolve_worktree_base() {
  if [[ -n "${SHIP_WORKTREE_DIR:-}" ]]; then echo "$SHIP_WORKTREE_DIR"; return; fi
  local cfg=""
  if command -v ship &>/dev/null; then
    cfg="$(ship config get worktrees.dir 2>/dev/null | tail -n1 | tr -d '[:space:]')"
  fi
  if [[ -n "$cfg" && "$cfg" != "(unset)" ]]; then echo "$cfg"; return; fi
  echo "${HOME}/dev/ship-worktrees"
}
WORKTREE_BASE="$(resolve_worktree_base)"
WORKTREE_BASE="${WORKTREE_BASE/#\~/$HOME}"
BASE_BRANCH=""
NO_OPEN=false
DRY_RUN=false
CONFIRM="${SHIP_DISPATCH_CONFIRM:-}"
SLUG=""
AGENT=""
SPEC_FILE=""
SHIP_AGENT_MODEL="${SHIP_AGENT_MODEL:-}"
EXTRA_SKILLS=()

# ── Parse args ────────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --slug)     SLUG="$2"; shift 2 ;;
    --agent)    AGENT="$2"; shift 2 ;;
    --spec)     SPEC_FILE="$2"; shift 2 ;;
    --base)     BASE_BRANCH="$2"; shift 2 ;;
    --dir)      WORKTREE_BASE="$2"; shift 2 ;;
    --model)    SHIP_AGENT_MODEL="$2"; shift 2 ;;
    --skill)    EXTRA_SKILLS+=("$2"); shift 2 ;;
    --no-open)  NO_OPEN=true; shift ;;
    --dry-run)  DRY_RUN=true; shift ;;
    --confirm)  CONFIRM=1; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$SLUG" || -z "$AGENT" || -z "$SPEC_FILE" ]]; then
  echo "Usage: dispatch.sh --slug <name> --agent <agent> --spec <file>" >&2
  echo "  --base <branch>   Branch to fork from (default: current)" >&2
  echo "  --dir <path>      Worktree base dir (default: ~/dev/ship-worktrees)" >&2
  echo "  --skill <id>      Extra skill to inject at compile time (repeatable)" >&2
  echo "  --no-open         Print launch command instead of opening terminal" >&2
  echo "  --confirm         Show spec and ask y/n before launching" >&2
  echo "  --dry-run         Show what would happen" >&2
  exit 1
fi

if [[ ! -f "$SPEC_FILE" ]]; then
  echo "Spec file not found: $SPEC_FILE" >&2
  exit 1
fi

WORKTREE_PATH="${WORKTREE_BASE}/${SLUG}"
BRANCH_NAME="job/${SLUG}"
BASE_BRANCH="${BASE_BRANCH:-$(git rev-parse --abbrev-ref HEAD)}"

# ── Detect terminal ───────────────────────────────────────────────────────────
detect_terminal() {
  local term="${SHIP_DEFAULT_TERMINAL:-}"
  if [[ -n "$term" ]]; then echo "$term"; return; fi
  if [[ -n "${WT_SESSION:-}" ]]; then echo "wt"; return; fi
  if [[ -n "${TMUX:-}" ]]; then echo "tmux"; return; fi
  case "${TERM_PROGRAM:-}" in
    iTerm.app)       echo "iterm" ;;
    WarpTerminal)    echo "warp" ;;
    vscode)          echo "vscode" ;;
    Apple_Terminal)  echo "manual" ;;
    *)               echo "manual" ;;
  esac
}

TERMINAL=$(detect_terminal)

# ── Dry run ───────────────────────────────────────────────────────────────────
if $DRY_RUN; then
  echo "dispatch (dry run):"
  echo "  slug:            $SLUG"
  echo "  agent:           $AGENT"
  echo "  spec:            $SPEC_FILE"
  echo "  base:            $BASE_BRANCH"
  echo "  worktree:        $WORKTREE_PATH"
  echo "  branch:          $BRANCH_NAME"
  echo "  terminal:        $TERMINAL"
  echo "  extra skills:    ${EXTRA_SKILLS[*]:-(none)}"
  echo "  SHIP_GLOBAL_DIR: $SHIP_GLOBAL_DIR"
  exit 0
fi

# ── Confirm ───────────────────────────────────────────────────────────────────
if [[ -n "$CONFIRM" ]]; then
  echo "=== Job Spec ==="
  cat "$SPEC_FILE"
  echo ""
  echo "Agent: $AGENT | Worktree: $WORKTREE_PATH | Branch: $BRANCH_NAME"
  read -rp "Dispatch? [y/N] " answer
  if [[ ! "$answer" =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
  fi
fi

# ── Create worktree (idempotent) ──────────────────────────────────────────────
mkdir -p "$WORKTREE_BASE"

if [[ -d "$WORKTREE_PATH" ]]; then
  echo "Worktree already exists: $WORKTREE_PATH"
else
  echo "Creating worktree: $WORKTREE_PATH (branch: $BRANCH_NAME from $BASE_BRANCH)"
  git worktree add "$WORKTREE_PATH" -b "$BRANCH_NAME" "$BASE_BRANCH"
fi

# ── Write job spec ────────────────────────────────────────────────────────────
DEST_SPEC="$WORKTREE_PATH/.artifacts/job-spec.md"
mkdir -p "$(dirname "$DEST_SPEC")"
cp "$SPEC_FILE" "$DEST_SPEC"
echo "Job spec written: $DEST_SPEC"

# ── Environment setup (hard stop) ─────────────────────────────────────────────
export SHIP_GLOBAL_DIR

if ! command -v ship &>/dev/null; then
  echo "Error: ship CLI not found. Install ship and ensure it is on PATH." >&2
  exit 1
fi

WITH_ARGS=()
for s in "${EXTRA_SKILLS[@]}"; do WITH_ARGS+=(--with "$s"); done

echo "Compiling agent: ship use $AGENT ${WITH_ARGS[*]} (SHIP_GLOBAL_DIR=$SHIP_GLOBAL_DIR)"
if ! (cd "$WORKTREE_PATH" && SHIP_GLOBAL_DIR="$SHIP_GLOBAL_DIR" ship use "$AGENT" --path "$WORKTREE_PATH" "${WITH_ARGS[@]}"); then
  echo "Error: 'ship use $AGENT' failed in $WORKTREE_PATH. Agent not launched." >&2
  exit 1
fi

# ── Verify MCP config ─────────────────────────────────────────────────────────
MCP_JSON="$WORKTREE_PATH/.mcp.json"
if [[ ! -f "$MCP_JSON" ]]; then
  echo "Error: .mcp.json not found in $WORKTREE_PATH after 'ship use'. Agent not launched." >&2
  exit 1
fi
if ! (grep -q '"command".*"ship"' "$MCP_JSON" && grep -q '"mcp"' "$MCP_JSON" && grep -q '"serve"' "$MCP_JSON"); then
  echo "Error: .mcp.json missing ship mcp serve config. Agent not launched." >&2
  exit 1
fi
echo "MCP config verified: $MCP_JSON"

# ── Worktree dependency bootstrap ─────────────────────────────────────────────
# Fresh worktrees ship empty node_modules and missing generated assets. Pre-stage
# them here so the agent doesn't hit `bun install` (which can rewrite the
# lockfile if there's drift) or a Rust build that include_str!s missing bundles.
if [[ -f "$WORKTREE_PATH/bun.lock" ]]; then
  echo "Installing dependencies (bun install --frozen-lockfile)"
  if ! (cd "$WORKTREE_PATH" && bun install --frozen-lockfile 2>&1 | tail -5); then
    echo "Warning: bun install --frozen-lockfile failed; agent may need to fix lockfile drift" >&2
  fi

  # Fast-path: copy gitignored generated bundles from the launching worktree.
  # Every file below is gitignored (regeneratable) but shipd `include_str!`s
  # most of them at compile time, so `cargo build` panics without them. We
  # copy from the launch repo because it's instant and almost always populated
  # by the developer's running daemon; the slow rebuild path is a fallback.
  LAUNCH_REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
  if [[ -n "$LAUNCH_REPO_ROOT" ]]; then
    # Lit `<ship-*>` bundles emitted by `bun run --filter '@ship/set' build` and embedded
    # by apps/shipd/src/views/assets.rs.
    if [[ -d "$LAUNCH_REPO_ROOT/apps/shipd/assets" ]]; then
      mkdir -p "$WORKTREE_PATH/apps/shipd/assets"
      cp "$LAUNCH_REPO_ROOT"/apps/shipd/assets/ship-*.js "$WORKTREE_PATH/apps/shipd/assets/" 2>/dev/null || true
    fi
    # SUI framework bundles + view bundles + lucide registry, emitted by the
    # in-daemon Bun worker and `bun run --filter '@ship/ui-components' build`. Embedded
    # by crates/deck/deck-server/src/assets.rs; build.rs panics if any are
    # missing or empty.
    if [[ -d "$LAUNCH_REPO_ROOT/crates/deck/deck-server/assets" ]]; then
      mkdir -p "$WORKTREE_PATH/crates/deck/deck-server/assets/sui-components" \
               "$WORKTREE_PATH/crates/deck/deck-server/assets/views"
      cp "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/sui-*.js \
         "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/sui-custom-elements.json \
         "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/lucide-icons.json \
         "$WORKTREE_PATH/crates/deck/deck-server/assets/" 2>/dev/null || true
      # Code-split component bundles — sui-components/ replaced the old
      # sui-islands/ dir after the islands-rename merge. Copy whichever
      # one(s) exist in the launch repo (defensive against half-migrated
      # checkouts).
      if [[ -d "$LAUNCH_REPO_ROOT/crates/deck/deck-server/assets/sui-components" ]]; then
        cp -r "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/sui-components/. \
              "$WORKTREE_PATH/crates/deck/deck-server/assets/sui-components/" 2>/dev/null || true
      fi
      if [[ -d "$LAUNCH_REPO_ROOT/crates/deck/deck-server/assets/sui-islands" ]]; then
        mkdir -p "$WORKTREE_PATH/crates/deck/deck-server/assets/sui-islands"
        cp -r "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/sui-islands/. \
              "$WORKTREE_PATH/crates/deck/deck-server/assets/sui-islands/" 2>/dev/null || true
      fi
      cp -r "$LAUNCH_REPO_ROOT"/crates/deck/deck-server/assets/views/. \
            "$WORKTREE_PATH/crates/deck/deck-server/assets/views/" 2>/dev/null || true
    fi
    # SDK manifests emitted by `cargo run -p xtask -- gen-sdk`, embedded by
    # apps/shipd/src/routes/sdk_manifest.rs.
    if [[ -d "$LAUNCH_REPO_ROOT/packages/deck/sdk/src/generated" ]]; then
      mkdir -p "$WORKTREE_PATH/packages/deck/sdk/src/generated"
      cp "$LAUNCH_REPO_ROOT"/packages/deck/sdk/src/generated/sdk-*.json \
         "$WORKTREE_PATH/packages/deck/sdk/src/generated/" 2>/dev/null || true
    fi
    # tree-sitter outputs — generated by `tree-sitter generate`, required by
    # packages/sui/grammar's Rust build.rs.
    if [[ -d "$LAUNCH_REPO_ROOT/packages/sui/grammar/src" ]]; then
      mkdir -p "$WORKTREE_PATH/packages/sui/grammar/src"
      cp "$LAUNCH_REPO_ROOT"/packages/sui/grammar/src/parser.c \
         "$LAUNCH_REPO_ROOT"/packages/sui/grammar/src/grammar.json \
         "$LAUNCH_REPO_ROOT"/packages/sui/grammar/src/node-types.json \
         "$WORKTREE_PATH/packages/sui/grammar/src/" 2>/dev/null || true
    fi
    echo "Copied generated artifacts from $LAUNCH_REPO_ROOT"
  fi

  # Rebuild fallback: if any required artifact still missing, regenerate it
  # locally so a fresh clone (no launch repo) still boots cleanly.
  if [[ ! -s "$WORKTREE_PATH/crates/deck/deck-server/assets/sui-runtime.js" ]]; then
    echo "Regenerating SUI framework bundles (slow path; launch-repo copy was unavailable)"
    (cd "$WORKTREE_PATH" && \
       bun run --filter '@ship/sui-runtime' bundle && \
       bun run --filter '@ship/sui-dom' bundle && \
       bun run --filter '@ship/sdk-client' bundle && \
       bun run --filter '@ship/ui-components' build && \
       bun run --filter '@ship/set' build && \
       bun run --filter '@ship/set' icons) 2>&1 | tail -20
  fi
  if [[ ! -s "$WORKTREE_PATH/packages/sui/grammar/src/parser.c" ]]; then
    echo "Regenerating tree-sitter parser.c"
    (cd "$WORKTREE_PATH/packages/sui/grammar" && bunx tree-sitter generate) 2>&1 | tail -10
  fi
  # sdk-urls.json is the sentinel — sdk-macros.json was retired in the
  # j2-vestigials sweep (macros pipeline deleted). Regenerate only when
  # urls is missing AND we couldn't copy from the launch repo.
  if [[ ! -s "$WORKTREE_PATH/packages/deck/sdk/src/generated/sdk-urls.json" ]]; then
    echo "Regenerating SDK manifests (cargo run -p xtask -- gen-sdk)"
    (cd "$WORKTREE_PATH" && cargo run -p xtask --quiet -- gen-sdk) 2>&1 | tail -10
  fi
fi

# ── Detect provider CLI ────────────────────────────────────────────────────────
PROVIDER_CLI="${SHIP_PROVIDER_CLI:-}"
if [[ -z "$PROVIDER_CLI" ]]; then
  # Read the first provider from the compiled agent config
  if [[ -f "$WORKTREE_PATH/CLAUDE.md" ]]; then
    PROVIDER_CLI="claude"
  elif [[ -f "$WORKTREE_PATH/AGENTS.md" ]]; then
    PROVIDER_CLI="codex"
  elif [[ -f "$WORKTREE_PATH/GEMINI.md" ]]; then
    PROVIDER_CLI="gemini"
  elif [[ -d "$WORKTREE_PATH/.cursor" ]]; then
    PROVIDER_CLI="cursor"
  elif [[ -f "$WORKTREE_PATH/.opencode.json" ]] || [[ -d "$WORKTREE_PATH/.opencode" ]]; then
    PROVIDER_CLI="opencode"
  else
    PROVIDER_CLI="claude"  # fallback
  fi
fi

# ── Auto-kickstart via tmux ───────────────────────────────────────────────────
# Polls a named tmux session for the Claude channels confirmation prompt, accepts
# it, then sends the job autostart message. No-op for other providers (no prompt).
# Runs in background — never blocks dispatch.
auto_kickstart_tmux() {
  local session="$1"
  (
    for i in $(seq 1 15); do
      sleep 1
      if tmux capture-pane -t "$session" -p 2>/dev/null | grep -q "I am using this for local development"; then
        # C-m (literal CR), not Enter — claude's menu widget does not respond
        # to the tmux Enter key name. Verified experimentally against a stuck
        # session where `Enter` was a no-op and `C-m` advanced past the menu.
        tmux send-keys -t "$session" C-m
        sleep 1
        # Send text and the submit key in SEPARATE send-keys invocations with
        # a pause between. Combined into one call, Claude Code's paste handler
        # collapses the trailing CR into the input buffer instead of firing
        # submit — observed across multiple dispatches where the user had to
        # press Enter manually. Splitting mimics human typing and submits.
        tmux send-keys -t "$session" "Read .artifacts/job-spec.md and execute it autonomously."
        sleep 1
        tmux send-keys -t "$session" Enter
        exit 0
      fi
    done
    # Prompt not found (non-claude provider or --dangerously-skip-permissions accepted it) —
    # still send the autostart message (same split-send pattern).
    tmux send-keys -t "$session" "Read .artifacts/job-spec.md and execute it autonomously."
    sleep 1
    tmux send-keys -t "$session" Enter
  ) &
}

# ── Open terminal ─────────────────────────────────────────────────────────────
# Support matrix (terminal × provider):
#
#   Strategy  | Auto-accept channels | Auto-kickstart | Notes
#   ----------|----------------------|----------------|-----------------------------
#   tmux      | yes (send-keys)      | yes            | Fully automated
#   wt        | yes (send-keys)      | yes            | WSL + Windows Terminal only
#   iterm     | no (AppleScript)     | no             | User must accept + start
#   vscode    | no (opens folder)    | no             | User must run provider CLI
#   warp      | no (opens folder)    | no             | User must run provider CLI
#   manual    | no                   | no             | Prints launch command only
#
#   Non-claude providers (codex, gemini, opencode): no channels prompt, auto-kickstart
#   message still sent on tmux/wt strategies.
#
# Build launch command with provider-specific flags
LAUNCH_FLAGS=""
MODEL_FLAG=""
if [[ -n "${SHIP_AGENT_MODEL:-}" ]]; then
  MODEL_FLAG="--model ${SHIP_AGENT_MODEL}"
fi
if [[ "$PROVIDER_CLI" == "claude" ]]; then
  LAUNCH_FLAGS="--dangerously-skip-permissions --dangerously-load-development-channels server:ship ${MODEL_FLAG}"
elif [[ "$PROVIDER_CLI" == "codex" ]]; then
  LAUNCH_FLAGS="${MODEL_FLAG}"
fi
LAUNCH_CMD="cd ${WORKTREE_PATH} && ${PROVIDER_CLI} ${LAUNCH_FLAGS}"

if $NO_OPEN; then
  echo ""
  echo "Launch manually:"
  echo "  $LAUNCH_CMD"
  exit 0
fi

case "$TERMINAL" in
  iterm)
    osascript -e "
      tell application \"iTerm2\"
        tell current window
          set newTab to (create tab with default profile)
          tell current session of newTab
            write text \"$LAUNCH_CMD\"
            set name to \"$SLUG\"
          end tell
        end tell
      end tell" 2>/dev/null || echo "iTerm2 AppleScript failed. Launch manually: $LAUNCH_CMD"
    ;;
  tmux)
    tmux new-window -d -n "$SLUG" "$LAUNCH_CMD" 2>/dev/null || { echo "tmux failed. Launch manually: $LAUNCH_CMD"; exit 1; }
    auto_kickstart_tmux "$SLUG"
    ;;
  warp)
    open -a Warp --args --working-directory "$WORKTREE_PATH" 2>/dev/null || echo "Warp launch failed. Launch manually: $LAUNCH_CMD"
    ;;
  vscode)
    code "$WORKTREE_PATH" 2>/dev/null || echo "VS Code launch failed. Launch manually: $LAUNCH_CMD"
    ;;
  wt)
    # From WSL: create a named tmux session so auto_kickstart_tmux can interact
    # with it via send-keys, then open a wt tab that attaches to it.
    # bash --login ensures ~/.bash_profile is sourced (PATH includes ~/.local/bin).
    WSL_DISTRO="${WSL_DISTRO_NAME:-Ubuntu}"
    tmux new-session -d -s "$SLUG" -c "$WORKTREE_PATH" 2>/dev/null || true
    tmux send-keys -t "$SLUG" "$PROVIDER_CLI $LAUNCH_FLAGS" Enter
    # `-w 0` targets the focused Windows Terminal window so new-tab
    # actually creates a tab in the existing wt window instead of
    # spawning a fresh wt window each dispatch.
    wt.exe -w 0 new-tab --title "$SLUG" -- wsl.exe -d "$WSL_DISTRO" bash --login -c "tmux attach-session -t $SLUG" 2>/dev/null || \
      echo "Windows Terminal launch failed. Attach manually: tmux attach-session -t $SLUG"
    auto_kickstart_tmux "$SLUG"
    ;;
  *)
    echo ""
    echo "Launch in a new terminal:"
    echo "  $LAUNCH_CMD"
    ;;
esac

echo ""
echo "Dispatched: $SLUG → $AGENT @ $WORKTREE_PATH"
