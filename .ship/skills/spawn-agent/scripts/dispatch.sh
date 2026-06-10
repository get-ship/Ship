#!/usr/bin/env bash
# Ship dispatch — create a worktree, write a job spec, compile the agent, open a terminal.
# Usage: bash scripts/dispatch.sh --slug <name> --agent <agent> --spec <file> [options]
#   --batch <name>   batch this dispatch into a shared wt window named
#                    "ship-batch-<name>"; first call opens the window, each
#                    subsequent call with the same name adds a split-pane.
#                    Lets you monitor N parallel agents in one wt window
#                    instead of N tabs. Lock expires after 24h.
# Env:
#   WT_PROFILE       Windows Terminal profile name (default: $WSL_DISTRO_NAME).
#                    Set to your preferred profile (e.g. "Ubuntu") so panes
#                    inherit your font/colors instead of wt's default profile.
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
SHIP_PROVIDER_CLI="${SHIP_PROVIDER_CLI:-}"
EXTRA_SKILLS=()
BATCH_NAME=""
NOBOOTSTRAP=false

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
    --batch)    BATCH_NAME="$2"; shift 2 ;;
    --harness)  SHIP_PROVIDER_CLI="$2"; shift 2 ;;
    --no-bootstrap) NOBOOTSTRAP=true; shift ;;
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
  echo "  --harness <cli>   Provider CLI to launch: claude (default) | omp | codex | ..." >&2
  echo "  --no-bootstrap    Skip build-env provisioning (deps/bundles); for self-contained jobs" >&2
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

# ── Install shared pre-push hook (blocks job/* branches) ──────────────────────
# Dispatched agents run with bypassPermissions so they don't pause for
# routine work — that means the only thing standing between them and a
# remote push is git itself.
#
# Git's default for worktrees is to use the MAIN repo's hooks dir
# (git-common-dir/hooks), not the per-worktree one. So we install one
# shared hook at <main>/.git/hooks/pre-push that gates by ref name:
# any push of refs/heads/job/* is refused. Operator pushes from main /
# feature branches are unaffected.
#
# Bypass with SHIP_ALLOW_PUSH=1 git push ... (deliberate, one-shot).
COMMON_GITDIR="$(git -C "$WORKTREE_PATH" rev-parse --git-common-dir)"
HOOK_DIR="$COMMON_GITDIR/hooks"
mkdir -p "$HOOK_DIR"
HOOK_FILE="$HOOK_DIR/pre-push"
HOOK_MARK="# ship-dispatch:pre-push v1"
if [[ ! -f "$HOOK_FILE" ]] || ! grep -q "$HOOK_MARK" "$HOOK_FILE" 2>/dev/null; then
  cat > "$HOOK_FILE" <<'PREPUSH'
#!/usr/bin/env bash
# ship-dispatch:pre-push v1
# Blocks pushes of refs/heads/job/* — those are dispatched agent branches
# the operator gates locally. Operator bypass: SHIP_ALLOW_PUSH=1 git push ...
if [[ -n "${SHIP_ALLOW_PUSH:-}" ]]; then exit 0; fi
while read -r local_ref local_sha remote_ref remote_sha; do
  case "$remote_ref" in
    refs/heads/job/*)
      echo "" >&2
      echo "  ✖ git push blocked — $remote_ref is a dispatched agent branch." >&2
      echo "    Operator gates locally and merges from the main repo." >&2
      echo "    Intentional push: SHIP_ALLOW_PUSH=1 git push ..." >&2
      echo "" >&2
      exit 1 ;;
  esac
done
exit 0
PREPUSH
  chmod +x "$HOOK_FILE"
  echo "Installed shared pre-push hook: $HOOK_FILE"
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
if [[ "$NOBOOTSTRAP" == true ]]; then
  echo "Skipping build-env provisioning (--no-bootstrap)"
elif [[ -f "$WORKTREE_PATH/bun.lock" ]]; then
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
    # deck's OWN baked product — sui-runtime/dom/sdk-client + the primitives
    # umbrella. Canonical bake target is now .deck/framework/ (Recommendation B1).
    # build.rs copies .deck/framework/sui-*.js → crates/deck/deck-core/assets/
    # (the include_str! staging path) at compile time. We only need to ensure
    # .deck/framework/ is populated, which the .deck/ copy below handles.
    # No separate crate-assets copy needed — build.rs owns that step.
    # deck BUILD output (.deck/) — app/user view bundles + code-split chrome
    # islands, read at runtime by deck-core's read_deck_asset. Not baked, so
    # not required for `cargo build`, but copying it spares the fresh worktree a
    # cold bundler pass before chrome renders.
    if [[ -d "$LAUNCH_REPO_ROOT/.deck" ]]; then
      mkdir -p "$WORKTREE_PATH/.deck"
      cp -r "$LAUNCH_REPO_ROOT"/.deck/. "$WORKTREE_PATH/.deck/" 2>/dev/null || true
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

  # Rebuild fallback: if .deck/framework/sui-runtime.js is still missing,
  # regenerate via the canonical bundler (cargo run -p deck-cli -- build-assets).
  # The .deck/ copy above handles the fast path; this is the cold-clone fallback.
  if [[ ! -s "$WORKTREE_PATH/.deck/framework/sui-runtime.js" ]]; then
    echo "Regenerating SUI framework bundles (slow path; launch-repo copy was unavailable)"
    (cd "$WORKTREE_PATH" && \
       cargo build -p sui-compiler && \
       cargo run -p deck-cli --quiet -- build-assets) 2>&1 | tail -20
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

  # Build the SUI compiler binaries into this worktree's target/. Without this,
  # the daemon + per-edit harness fall back to the stale installed
  # ~/.ship/runtime/sui-compiler instead of the worktree's current source, so a
  # fresh worktree silently lints/compiles against old language rules. Also warm
  # the lib-only lint example (the CLI form agents call per edit). Robust by
  # design: offline / locked-registry builds must not abort the dispatch — log
  # and continue; the agent's own edit-loop will surface a missing binary.
  echo "Building SUI compiler binaries (cargo build -p sui-compiler -p sui-compiler-napi)"
  if ! (cd "$WORKTREE_PATH" && cargo build -p sui-compiler -p sui-compiler-napi 2>&1 | tail -15); then
    echo "Warning: cargo build of sui-compiler/sui-compiler-napi failed (offline?); agent must build before its edit loop" >&2
  fi
  echo "Warming the sui-lint check example (cargo build -p sui-lint --example check)"
  if ! (cd "$WORKTREE_PATH" && cargo build -p sui-lint --example check 2>&1 | tail -10); then
    echo "Warning: building the sui-lint check example failed (offline?); agent must build before its edit loop" >&2
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
  local provider="${PROVIDER_CLI:-claude}"
  (
    # Wait for the provider to be READY for input, THEN send the kickoff (split
    # text → pause → Enter; a combined send collapses the CR into the buffer).
    #   claude    → the channels-confirm prompt (accept with C-m, not Enter).
    #   omp/other → the Pi CLI prompt box (the `π` glyph). omp shows NO channels
    #               prompt and can take >15s to boot its TUI, so the old fixed
    #               15s delay fired into a not-yet-ready input. Poll up to 60s.
    for i in $(seq 1 60); do
      sleep 1
      pane="$(tmux capture-pane -t "$session" -p 2>/dev/null)"
      if [[ "$provider" == "claude" ]]; then
        if grep -q "I am using this for local development" <<<"$pane"; then
          tmux send-keys -t "$session" C-m   # advance past the channels menu
          sleep 1
          break
        fi
      else
        if grep -qF 'π' <<<"$pane"; then      # Pi CLI prompt rendered ⇒ ready
          break
        fi
      fi
    done
    # Best-effort even if readiness never showed: send the autostart message.
    tmux send-keys -t "$session" "Read .artifacts/job-spec.md and execute it autonomously."
    sleep 1
    tmux send-keys -t "$session" Enter
  ) &
  # Detach from the parent's job table so the background poller survives
  # dispatch.sh exiting. Without disown, when the script returns its parent
  # (e.g. a script-launcher) can deliver SIGHUP to backgrounded children and
  # kill the kickstart before claude's menu finishes rendering — observed on
  # the 3rd agent in a batch where the parent exits faster than the kickstart
  # has had time to detect the prompt.
  disown
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
  # bypassPermissions: agent never pauses to ask the operator (they're not
  # at the keyboard). Network/remote operations are gated at the git layer
  # by the per-worktree pre-push hook installed above — bypassPermissions
  # at the claude level + pre-push at the git level layer correctly:
  # routine work runs unattended, push is refused without operator intent.
  #
  # Tried --permission-mode auto first; too cautious — agents paused on
  # routine actions the operator couldn't approve, and the work stalled.
  PERMISSION_MODE="${SHIP_PERMISSION_MODE:-bypassPermissions}"
  LAUNCH_FLAGS="--permission-mode ${PERMISSION_MODE} --dangerously-load-development-channels server:ship ${MODEL_FLAG}"
elif [[ "$PROVIDER_CLI" == "codex" ]]; then
  LAUNCH_FLAGS="${MODEL_FLAG}"
elif [[ "$PROVIDER_CLI" == "omp" ]]; then
  # omp (Pi CLI) reads CLAUDE.md + .claude/skills (claude's compiled config),
  # so no new compile harness is needed. --approval-mode yolo runs unattended
  # (the claude bypassPermissions equivalent); the per-worktree pre-push hook
  # still gates network/remote ops. Model is omp's own configured default
  # unless --model is passed.
  LAUNCH_FLAGS="--approval-mode yolo ${MODEL_FLAG}"
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
    # with it via send-keys, then open a wt tab/pane that attaches to it.
    # bash --login ensures ~/.bash_profile is sourced (PATH includes ~/.local/bin).
    WSL_DISTRO="${WSL_DISTRO_NAME:-Ubuntu}"
    # `-p <profile>` selects the user's preferred Windows Terminal profile
    # (font/colors/etc.) instead of wt's default. Defaults to the distro
    # name (matches the convention `WT_PROFILE=Ubuntu` for an Ubuntu WSL
    # profile). Override with WT_PROFILE=<name> in the environment.
    WT_PROFILE="${WT_PROFILE:-$WSL_DISTRO}"
    tmux new-session -d -s "$SLUG" -c "$WORKTREE_PATH" 2>/dev/null || true
    tmux send-keys -t "$SLUG" "$PROVIDER_CLI $LAUNCH_FLAGS" Enter

    if [[ -n "$BATCH_NAME" ]]; then
      # Batching: every dispatch with the same --batch NAME lands in one
      # wt window (so a single tab logically holds N agent panes — easier
      # to monitor 4-5 parallel agents at once). First call opens the
      # window + first pane; subsequent calls split-pane into it via
      # `-w ship-batch-<NAME>` window targeting. Marker file in /tmp
      # decides first-vs-subsequent.
      WT_WINDOW="ship-batch-${BATCH_NAME}"
      BATCH_LOCK="/tmp/.ship-dispatch-batch-${BATCH_NAME}.lock"
      if [[ -f "$BATCH_LOCK" ]] && (( $(date -r "$BATCH_LOCK" +%s 2>/dev/null || echo 0) > $(($(date +%s) - 86400)) )); then
        # Same batch within 24h → split-pane in the batch window.
        wt.exe -w "$WT_WINDOW" split-pane -V -p "$WT_PROFILE" wsl.exe -d "$WSL_DISTRO" bash --login -c "tmux attach-session -t $SLUG" 2>/dev/null || \
          echo "Windows Terminal split-pane failed. Attach manually: tmux attach-session -t $SLUG"
      else
        # New batch (or stale lock) → new window with the first pane.
        wt.exe -w "$WT_WINDOW" new-tab --title "$BATCH_NAME" -p "$WT_PROFILE" wsl.exe -d "$WSL_DISTRO" bash --login -c "tmux attach-session -t $SLUG" 2>/dev/null || \
          echo "Windows Terminal launch failed. Attach manually: tmux attach-session -t $SLUG"
        touch "$BATCH_LOCK"
      fi
    else
      # `-w 0` targets the focused Windows Terminal window so new-tab
      # actually creates a tab in the existing wt window instead of
      # spawning a fresh wt window each dispatch.
      wt.exe -w 0 new-tab --title "$SLUG" -p "$WT_PROFILE" wsl.exe -d "$WSL_DISTRO" bash --login -c "tmux attach-session -t $SLUG" 2>/dev/null || \
        echo "Windows Terminal launch failed. Attach manually: tmux attach-session -t $SLUG"
    fi
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
