# Framework Issues & Data Requests

**Operating principle.** We build the hub on the RPC surface **we have**. We do *not*
change the RPC surface to make ported shell code work — the shell was built on a
now-deprecated surface, and "a prior data call is missing → add it to the backend"
is the instinct that has wrecked frontend work before. When a surface has a
*legitimate* need for an operation or data that isn't available, it is logged here
as a **REQUEST** for the operator to validate — never silently added.

Two kinds of entries:
- **FW-n — framework bugs.** Real defects in deck/sui/shipd behavior. Fixable; not RPC-surface changes.
- **REQ-n — data/op requests.** A surface wants an op/shape not on the current contract. **Blocked pending operator validation. Do not add to the backend to satisfy ported code.**

---

## FW-1 · deck router: static routes mis-route to `:param` siblings
- **Symptom:** `/studio/agents/create` resolves to the `:id` detail view (mounts the wrong controller → was the source of the `agent is not defined` crash).
- **Cause:** `crates/deck/deck-bundler/src/routes.rs:871` sorts patterns lexicographically (`a.pattern.cmp(&b.pattern)`); `:id` (0x3A) sorts before `create` (0x63), and the router (`router/src/index.ts:353`) first-matches without specificity.
- **Blast radius:** any static route that is a sibling of a `:param` route — not just create.
- **Fix:** sort static segments before dynamic, or match by specificity.
- **Location:** **sui repo** (deck-bundler + router). **Status: blocked on sui access (operator ping).**

## FW-2 · component catalog gathers by relative path (dead end)
- Old `workspace.json` / `/_deck/catalog` walked the workspace by relative path; broken cross-repo and in prod. The **registry** is the sanctioned replacement (no relative-path component gathering). Investment when ready; `/studio/components` stays empty until then.

---

## REQ-1 · agent detail / settings / wire / create-context data  — NEEDS OPERATOR VALIDATION
- **Surfaces (ported from shell):** `/studio/agents/[id]`, `[id]/settings`, `[id]/wire`, `create`.
- **Ops the shell used:** `agents.wire`, `agents.settings`, `agents.create_ctx` (live/sse).
- **Current contract (registered live resolvers):** `agents.summaries`, `collections.list`, `account.state`, `git.overview` — the agent-detail ops are **not** on it.
- **Question for the operator (do NOT pre-answer by editing the backend):** Are these agent sub-surfaces in scope on the current contract? If yes, what is the right shape on the *available* surface (and unary vs live)? Until validated, these routes render chrome + an empty/error state. The ported code currently assumes the deprecated surface.

---

### Notes
- icon-404s: a per-page asset 404 (lucide). Cosmetic; lower priority. Likely framework asset-serving, not a data issue.
