---
name: working-in-helm
audience: internal
stable-id: working-in-helm
description: The happy path for any agent working through Helm. How intent and reality meet — read your intent, prove each clause with a test, report findings as claims (and clear them), coordinate, and keep the codebase a reflection of the present. Read this first.
tags: [helm, workflow, methodology, core]
authors: [ship]
---

# Working in Helm

You are a dispatched agent. **Helm is how you receive intent and report reality.** It is a shared workspace, not a dump. Use it correctly — the next agent (and the operator) reads what you leave.

The words below are precise. When in doubt, read the glossary (`get_wiki`) — it is the source of truth for the vocabulary. Do not invent synonyms; one drifting word breaks shared understanding.

## The model

- **System → Component → Feature** — the intent hierarchy. Your work targets a Component or a Feature.
- **Clause** — the provable unit of intent: a **requirement** (something the unit must do) or an **invariant** (a rule that must always hold). Shape: a statement, its declared evidence, and a `gap` flag. *(In the schema a clause is a `condition` — same thing.)*
- **Claim** — an **issue**: a bug, an observation, a request, or a HITL decision. A claim is a message in the inbox. **A claim is NOT proof and NOT a requirement.** Do not confuse it with a clause.
- **Proof** — **derived, never hand-set.** A clause is proven by a **run** (a test/bench/eval result) measured against its evidence. Status is observed: passing / failing / stale / untested / gap.

## Your job: align actual with intent

1. **Read your intent.** `get_component` / `list_features` / `list_gaps` — find the clauses your slice must satisfy. If the intent is unclear, that is a HITL claim, not a guess.
2. **Prove each clause with a test in the living suite.** Write the test that *states* the clause, watch it fail, make it pass, leave it. **The test is the only proof artifact.** Do not write a proof document, do not file a claim that says "done," do not leave a comment asserting it works. A clause is satisfied when a run proves it — nothing else counts.
3. **The gate is the suite, not your word.** The operator (or commander) re-runs it. **Never self-report "done."** Your report is the green suite.

## Claims are issues — file sparingly, and clear them

- File a claim (`file_claim`) only for a real **finding** (a bug or observation worth surfacing), a **blocker**, or a **HITL** decision you need. Pick the right `kind`: `bug` / `observation` / `request` / `hitl`.
- **Do not narrate your work as claims.** "I refactored X", "I added a test" — none of that. The diff and the suite are the record.
- **Clear what you handle.** A claim leaves the inbox one of two ways: `reconcile_claim` (substantive — kept with a verdict) or `delete_claim` (trivial, handled, or misfiled — cleared outright). An inbox that only grows is a dump bucket. Leave it smaller than you found it where you can.

## Coordinate — you are not alone

- `overlap` before you edit shared files — see which other active jobs touch them. Colliding fixes waste everyone's work.
- `list_active_agents` — see who is working and on what.
- `link_mission_session` — tie your session to its mission. Working notes go to the mission scratch (`write_mission_scratch`), never to loose files in the tree.

## Discipline — keep the codebase a reflection of the present

- **Comments describe the present, not the past.** Never leave `// this used to be X`, `// removed Y`, `// previously did Z`, or a changelog in a comment. **Git holds history.** When you delete code, delete it cleanly. A tombstone comment is noise the next reader has to disprove.
- **Be terse.** Match the surrounding code's comment density. Don't narrate the obvious. A comment earns its place by explaining *why*, not *what*.
- **No `.artifacts`. No dumping into `docs/`. A README is not documentation.** Scattered files rot because no one looks at them. Decisions, rationale, and findings go to **Helm** — a claim, the mission, or the wiki — where the team actually looks.
- **The repo is the current state, not an ode to a deprecated past.** If it's dead, remove it. If it mattered, it's in git and in Helm.

## In one line

Read intent → prove each clause with a test → surface findings as claims and clear them → coordinate → leave the code clean. That's the whole loop.
