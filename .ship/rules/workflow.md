# Workflow

The binding workflow. Full SSOT (model, surfaces, rationale): helm `data/wiki/workflow.md`;
layout decisions: helm `data/wiki/conventions.md`. This file is the enforceable subset.

* **One space.** `~/dev/Ship/` holds every repo. `~/dev/ship` + `~/dev/sui` are symlinks into it — never a second clone.
* **One worktree home.** `~/dev/Ship/worktrees/<repo>/<slug>` (set via `ship config worktrees.dir`). A worktree exists IFF a helm mission has a job for it — the mission dictates worktrees.
* **Branch flow: job → integration → main.** `job/<slug>` = one job, one worktree, branched from the mission's integration branch. **integration branch** = ONE long-lived branch per mission (e.g. `feat/sui-rebuild`); jobs squash-**LAND** onto it. `main` = stable; integration squash-**RELEASES** to main + tag, **operator-gated**. Long-lived feature branches are intended — do NOT release while work is in progress.
* **Mission on the long-lived feature branch.** The mission agent owns the integration branch; subagents work on `job/*` branches that the mission agent LANDS in.
* **Land-or-clean gate.** Every branch/worktree is ACTIVE (open job) | LANDED (squash-merged → branch + worktree DELETED) | CLEAN (no unique value → DELETED). Anything else is a leak. **Parked = lost or duplicated work** — never park.
* **It's us.** Operator-driven sessions + `omp` as compute for jobs. No "other agents" to tiptoe around — every branch is ours; land it or clean it.
* **Git via ship-git.** The land/release run on the GitService leaf (`VcsPort.land/release`), not raw `git` shell-out. Dispatch IS the helmd RPC `ship.ops.v1.AgentsService/Dispatch` (worktree + compile + harness); `dispatch-portable.sh` is the thin CLI client that POSTs it.
