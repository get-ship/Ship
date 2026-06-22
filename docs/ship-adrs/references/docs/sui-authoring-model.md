---
title: "SUI Authoring Model: Pure-Expression Templates + Controller Logic (Not JSX)"
description: "ADR: a `.sui` template carries declarative directives and PURE expressions (signal reads + pure transforms); capabilities (I/O, authority, non-determinism, non-owned mutation) and statements/declarations belong in `controller.ts`, reached through the ship SDK. The axis is purity + authority, not complexity. SUI is the SFC lineage, deliberately not JSX. Enforced by compile lint + client SES + server deny-loader."
group: ADRs
order: 41
audience: internal
status: accepted
date: 2026-06-21
supersedes: []
tags: [sui, authoring, templates, controller, lint, security, ssr]
---

# SUI Authoring Model: Pure-Expression Templates + Controller Logic (Not JSX)

## Context

SUI splits a view into a `.sui` template and an optional paired `controller.ts`
([SUI / Deck crate boundary](./sui-deck-crate-boundary.md) names the framework;
the [SUI architecture doc](../../../ship-sui/references/docs/index.md) names the
three roles). As the framework's authoring power grows through the cutover, there
is steady pressure to let templates do "just a little more" — an inline `fetch`,
a local `const`, an in-place `arr.sort()`. Each looks harmless in isolation; in
aggregate they would dissolve the boundary that the renderer, the SSR walker, the
LSP, and the capability gate all depend on.

Two framings get this wrong:

- **"Heavy vs. light."** Tempting but false. A 200-line *pure* transform chain is
  fine in a template; a one-line `fetch()` is not. Length is not the axis.
- **"It's basically JSX."** SUI is the Svelte/Vue/Astro single-file-component
  lineage. JSX admits arbitrary statements and expressions into markup; SUI does
  not, on purpose — that restriction is what keeps a view statically analyzable
  without running it.

The decision below locks the line so it stops being relitigated per-view and per
PR. It was settled in pairing on 2026-06-20 (`cutover-clarity.md` §1–§2).

## Decision

The authoring boundary has two halves. The axis is **purity + authority**, not
complexity.

### 1. The template line — pure expressions, not capabilities

A `.sui` template expression must be a **pure function of scope signals**: safe to
re-run on every dependency change, and identical when rendered on the server and
the client.

- **Allowed in `.sui`:** signal reads + pure transforms —
  `map`/`filter`/`reduce`/`slice`/arithmetic/string/ternary/comparison/
  member-access/`toFixed`.
- **Banned (→ `controller.ts`, via the ship SDK):** I/O & authority
  (`fetch`/`XHR`/`WebSocket`/`eval`/`Function`/`import()`/`localStorage`),
  non-determinism (`Date.now`/`Math.random`/`new Date()`), and mutation of
  non-owned state (`arr.sort()` → `[...arr].sort()`).

**Why here:** the renderer re-runs expressions on every dependency change (they
must be safe to re-run); authority flows ONLY through the ship SDK (the auth
gate); pure expressions SSR identically server + client.

### 2. Not JSX — expression-in-template vs. statement-in-controller

- **Template (`.sui`):** declarative directives (`<prop>`/`<data>`/`<client>` +
  `{#for}`/`{#if}`/`{#while-loading}`) + pure expressions.
- **Controller (`.ts`):** full JS — variable declarations,
  `signal`/`derived`/`live`/`effect`, the auth-gated I/O.

There are **no `const`/`let`/statements in a `.sui` file.** Want a local binding?
**Promote it to a `derived`** in the controller. The friction *is* the feature:
it forces every named computation to become a declared, typed, statically
analyzable member of the view's scope. Keeping the template to directives +
expressions is what makes props, data dependencies, and routes knowable without
running JS — which is what powers the LSP, route generation, and prop contracts,
and what keeps the template SSR-safe, capability-enforceable, and
agent-authorable. The cutover grows authoring power the right way: template
expressions compile to real TypeScript that `tsc` and the LSP type-check — not by
admitting statements.

### Enforcement (three independent layers)

1. **Compile lint** — `controller_no_ambient_authority` at Error severity rejects
   banned capabilities at build time.
2. **Client SES** — template expressions evaluate in a sandbox where `fetch` /
   `window` are `undefined`, so a slipped-through capability fails closed at
   runtime.
3. **Server deny-by-default loader** — the SSR side refuses ambient authority
   rather than allow-listing it.

The **template-subset lint** — the pure-expression half of the line, stated
positively, with instant author feedback — folds INTO the rust-eval cut (Phase 2
of `post-cutover-audit.md`). Until it lands, the same line is also encoded
mechanically in the `sui-eval` skill's grader (`scripts/template-purity.ts`,
wired into `grade.ts`), which fails the structural pass on any capability or
statement found in a `.sui` template. The author-facing teaching lives in
[the SUI authoring-model doc](../../../ship-sui/references/docs/authoring-model.md).

## Consequences

### Positive

- **SSR identity.** Pure template expressions produce the same output on the
  server and after client attach, so hydration markers line up byte-for-byte.
- **The auth gate holds.** Authority has exactly one path — the ship SDK. A
  template can never reach around it.
- **Static analyzability.** Props, data-deps, and routes are extractable without
  executing JS, which is the precondition for the LSP, route-gen, and prop
  contracts.
- **Re-run safety.** The reactive renderer can re-evaluate any expression freely;
  none of them fetch, mutate, or read the clock.
- **One contract, stated once.** Lint, SES sandbox, deny-loader, and the eval
  grader all enforce the same line — no parallel definitions to drift apart.

### Negative

- **Promotion friction.** A local binding that would be a one-line `const` in JSX
  must become a `derived` in the controller. This is deliberate, but it is a real
  ergonomic cost and an adjustment for authors arriving from React/JSX.
- **The positive lint isn't fully shipped yet.** Today the line is enforced by the
  capability lint + SES + deny-loader + the eval grader; the unified
  template-subset lint with instant feedback arrives with the Phase-2 cut. Until
  then, some author feedback is post-hoc (compile/eval) rather than in-editor.

## Alternatives Considered

- **Admit statements / JSX-style logic into templates.** Rejected: forfeits
  static analyzability (LSP, route-gen, prop contracts) and makes capability
  enforcement undecidable. The whole value of the directive set evaporates.
- **Draw the line at "heavy vs. light" code.** Rejected: wrong axis. It would
  both ban legitimate long pure chains and wave through a short `fetch()`.
- **Enforce at runtime only (SES sandbox).** Rejected: authors need build-time
  and in-editor feedback, and the server needs its own defense in depth — hence
  three layers, not one.
- **Leave it implicit / per-reviewer.** Rejected: the line was being relitigated
  per view; locking it as a contract is the point of this ADR.

## How to Measure Success

- No new `.sui` template introduces a banned capability or a statement/declaration
  (caught by lint + the `sui-eval` `template-purity` grader at score floor).
- The Phase-2 template-subset lint, when it lands, is a positive restatement of
  this same line — it adds author feedback, it does not change the contract.
