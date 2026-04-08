# Agent Workspace

## .ship/ is configuration — do not write to it

`.ship/` contains agents, skills, rules, and manifests. The only writers are `ship install` and `ship use`. Agents never create, modify, or delete files in `.ship/`.

## .artifacts/ is your scratchpad

All ephemeral working artifacts go in `.artifacts/`. It is gitignored and never committed.

| Artifact | Path |
|----------|------|
| Job spec | `.artifacts/job-spec.md` |
| Mockups | `.artifacts/mockup.html` |
| Screenshots | `.artifacts/design-spec/screenshots/` |
| Any working files | `.artifacts/<name>` |

Source code changes go on the branch, not in `.artifacts/`.

## Notes and ADRs are for humans

Notes and ADRs are human-facing documents. Agents do not write plans, coordination, or scratch work into notes. When asked, agents help humans draft and refine them.

Agent state goes elsewhere:
* Progress → `log_progress` / `append_job_log`
* Plans and specs → `.artifacts/` files
* Coordination → job queue (`create_job`, `update_job`)
* Handoffs → `complete_workspace` / `handoff.md` in worktree root
