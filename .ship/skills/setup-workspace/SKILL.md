---
name: setup-workspace
audience: internal
stable-id: setup-workspace
description: Configure a multi-agent terminal environment. Reads pod.md layout, opens one tab/window per agent in their worktree.
tags: [environment, terminal, workflow, setup]
authors: [ship]
---

# Workspace Setup

Launch your agent pod in one command. Reads `.artifacts/pod.md` and opens a
terminal tab for each agent in its worktree.

## Usage

```bash
bash .ship/skills/setup-workspace/scripts/setup.sh
```

Reads `.artifacts/pod.md` by default. Pass `--pod <file>` to use a different layout.

```bash
bash .ship/skills/setup-workspace/scripts/setup.sh --pod .artifacts/pod.md
bash .ship/skills/setup-workspace/scripts/setup.sh --dry-run  # preview without opening
```

## Pod layout file

Create `.artifacts/pod.md` to define your agent pod. The file is gitignored.

```yaml
agents:
  - name: rust
    agent: rust-lane
    path: ~/dev/Ship/worktrees/rust-work
  - name: web
    agent: web-lane
    path: ~/dev/Ship/worktrees/web-work
```

`path` defaults to `$SHIP_WORKTREE_DIR/<name>` if omitted.

Save the file once, reuse every session.

## Configuration

```bash
ship vars set setup-workspace terminal tmux        # force terminal type
ship vars set setup-workspace worktree_dir ~/dev/Ship/worktrees
```

Falls back to `SHIP_DEFAULT_TERMINAL` / `SHIP_WORKTREE_DIR` env vars if vars are not set.

| Var | Default | What |
|-----|---------|------|
| `terminal` | auto-detected | `tmux`, `wt`, `iterm`, `warp`, `manual` |
| `worktree_dir` | `~/dev/Ship/worktrees` | Base path for agent worktrees |

## Terminal support

| Terminal | Detection | Behavior |
|----------|-----------|----------|
| tmux | `$TMUX` set | `tmux new-window -d` per agent |
| Windows Terminal (WSL) | `$WT_SESSION` set | `wt.exe -w 0 nt` per agent |
| iTerm2 | `$TERM_PROGRAM=iTerm.app` | AppleScript tab per agent |
| manual | fallback | Prints launch commands |

Set `SHIP_DEFAULT_TERMINAL` to override detection. Useful when `WT_SESSION` is
set inside tmux (WSL) and you want tmux windows instead.

## Teardown

```bash
# tmux
tmux kill-session -t ship

# or close tabs manually — worktrees persist until gate passes or manual cleanup
git worktree remove ~/dev/Ship/worktrees/<slug>
git branch -d job/<slug>
```
