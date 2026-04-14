# Security Posture Audit — Review Checklist

Grading checklist for `.artifacts/security-posture.md`.
The reviewer (step 3) evaluates each item as Pass / Fail with evidence.

---

## 1. Source Coverage

Every file listed in the spec's "Required reading" must be cited at least
once with a `file:line` reference.

- [ ] `apps/shipd/src/auth.rs` cited with line reference
- [ ] `apps/shipd/src/pairing.rs` cited with line reference
- [ ] `apps/shipd/src/session.rs` cited with line reference
- [ ] `apps/shipd/src/attest.rs` cited with line reference
- [ ] `apps/shipd/src/scopes.rs` cited with line reference
- [ ] `apps/mcp/src/server/tool_gate.rs` cited with line reference
- [ ] `apps/mcp/src/server/role_gate_integration_tests.rs` cited with line reference
- [ ] `apps/mcp/src/server/tools/event.rs` cited with line reference
- [ ] `apps/shipd/src/view_sessions_api.rs` cited with line reference
- [ ] `apps/shipd/src/annotations_api.rs` cited with line reference
- [ ] `apps/shipd/src/mcp_apps_bridge.rs` cited with line reference
- [ ] `apps/shipd/src/views.rs` cited with line reference
- [ ] `apps/shipd/src/views_raw.rs` cited with line reference
- [ ] `apps/web/src/features/studio/PlatformProvider.tsx` cited with line reference
- [ ] `apps/web/src/lib/daemon-config.ts` cited with line reference
- [ ] `crates/core/runtime/src/events/permissions.rs` cited with line reference
- [ ] `crates/core/runtime/src/kernel_security.rs` cited with line reference

## 2. Surface Inventory Completeness

Each of the following auth surfaces must appear in the inventory with **all
five required attributes**: identity asserted, issuer, consumer, rotation /
expiry policy, failure mode.

| Surface | Identity | Issuer | Consumer | Rotation/Expiry | Failure Mode |
|---------|----------|--------|----------|-----------------|--------------|
| Daemon bearer token (`daemon.token`) | [ ] | [ ] | [ ] | [ ] | [ ] |
| Hook tokens (`ship_hook_*`) | [ ] | [ ] | [ ] | [ ] | [ ] |
| MCP session tokens (`ship_sess_*`) + scope model | [ ] | [ ] | [ ] | [ ] | [ ] |
| Browser session cookie (`ship_session` / `ship_browser_*`) | [ ] | [ ] | [ ] | [ ] | [ ] |
| PKCE pairing flow (code + claim token + paired session) | [ ] | [ ] | [ ] | [ ] | [ ] |
| Server-side attestation (JWT / EdDSA via `attest.rs`) | [ ] | [ ] | [ ] | [ ] | [ ] |
| View-session ownership (`view_id` scoped to agent) | [ ] | [ ] | [ ] | [ ] | [ ] |
| Annotation event auth (`POST /api/runtime/annotations`) | [ ] | [ ] | [ ] | [ ] | [ ] |
| MCP Apps CSP (`_meta.ui.csp` / iframe isolation) | [ ] | [ ] | [ ] | [ ] | [ ] |
| `--dev` mode relaxations | [ ] | [ ] | [ ] | [ ] | [ ] |
| Hook 401 token rotation on daemon restart | [ ] | [ ] | [ ] | [ ] | [ ] |
| `enforce_mode_tool_gate` (role-based tool gating) | [ ] | [ ] | [ ] | [ ] | [ ] |
| Event delivery scope filtering (Global/Workspace/Directed/Elevated) | [ ] | [ ] | [ ] | [ ] | [ ] |
| Reserved event namespace prefix enforcement | [ ] | [ ] | [ ] | [ ] | [ ] |
| Annotation rate limiting (per-agent token bucket) | [ ] | [ ] | [ ] | [ ] | [ ] |

A surface discovered during the audit that is not in this table is acceptable
**only if** the doc clearly marks it as "additionally discovered" and
provides all five attributes.

## 3. Drift and Gap Analysis

- [ ] At least **5** distinct drift/gap items are listed
- [ ] Each item has an explicit severity label: `blocker`, `major`, or `minor`
- [ ] Each item has a brief recommendation (at least one sentence)
- [ ] Items reference specific code locations or behaviors (not abstract)
- [ ] At least one item addresses cross-surface inconsistency (e.g., TTL
      mismatch between session types, scope model divergence)

## 4. Unification Proposal

- [ ] Proposal contains a prioritized list of moves (not just a flat set)
- [ ] Priority ordering is justified by blast radius or risk
- [ ] Each move has a clear "done" predicate (a testable condition, not a
      vague goal like "improve security")
- [ ] At least one move addresses token lifecycle consistency
- [ ] At least one move addresses scope model unification or simplification
- [ ] No move proposes code changes that conflict with the "out of scope"
      constraint (this is a design artifact, not an implementation PR)

## 5. Follow-up Spec Stubs

- [ ] At least **one** follow-up implementation spec stub exists
- [ ] Each stub includes: job title, scope description, acceptance criteria
- [ ] Stubs are scoped narrowly enough to be a single job (not a multi-week
      epic disguised as one spec)

## 6. No Code Modifications

- [ ] No files under `apps/` are modified (diff check: `git diff --name-only`)
- [ ] No files under `crates/` are modified
- [ ] No `.ts`, `.tsx`, `.rs`, `.jsonc` files are created or changed

## 7. Document Quality

- [ ] `.artifacts/security-posture.md` exists
- [ ] Document is under 1500 lines (`wc -l < 1500`)
- [ ] Valid markdown (no broken internal links, no unclosed formatting)
- [ ] All `file:line` references point to lines that actually exist in the
      current codebase (spot-check at least 5 references)
- [ ] Sections are clearly delineated (Inventory, Drift/Gaps, Unification
      Proposal, Follow-up Stubs)

---

## Grading

| Result | Criteria |
|--------|----------|
| **Pass** | All items in sections 1-7 checked, with evidence logged per item |
| **Fail** | Any unchecked item; reviewer must note which and why |

The reviewer logs evidence inline (e.g., "line 42 of security-posture.md
lists daemon bearer with identity=local-owner, issuer=startup, ...") for
each checkbox.
