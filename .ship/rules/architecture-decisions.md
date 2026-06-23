# Locked architectural decisions (do not re-litigate)

> `CLAUDE.md` is mode-compiled (`ship use` regenerates it), so durable decisions live HERE, not there. This file is the source of truth for the cross-session principles that keep getting re-taught.

## Multi-repo bounded contexts
Split at a **stable contract with an independent consumer**; do NOT split tightly-coupled internals (no contract = friction with no guardrail). Contract boundaries are agent guardrails: an agent in a repo can only touch that repo's contract surface (this is *why* we split — to stop the shipd/axum clobbering).

Target topology:
- **contract** — SDK + protos + types. The linchpin every repo pins.
- **ship** — daemon / backend / services. Implements the contract.
- **sui** — deck + UI stdlib + `sui-*` components (the framework). Consumed via the registry.
- **apps** — hub, spaces, site, docs + **`ship-*` components** (Ship's app/domain UI). Consume sui + the SDK. (`ship-*` do NOT belong in sui.)
- **cloud** — thin RPC service. Own repo when ready (cheap; thin, proto-only seam).
- **ship-ops** — peer consumer of sui; has its own backend.
- Do NOT split: L0↔L1 (internal layering — enforce with lint, not a repo), `cli`, `ship-code`.

## Components
- `sui-*` → sui; `ship-*` → apps.
- Consumed via the **registry** (the eventual mechanism), NEVER relative-path workspace-walking. **Rule: no relative paths to pull components from outside the repo.** The `workspace.json` / `/_deck/catalog` approach is dead — the registry replaces it.

## JS runtime / SSR
- `ship-js` (deno_core/V8) is the untrusted-execution core, including deck SSR (deck serves untrusted user/agent code in-process on axum — the isolate is a requirement, not accidental coupling).
- Toolkit-with-contracts, not one-runtime-everywhere: V8 (untrusted core) · bun/Hono (trusted fast SSR, in progress) · render-eval (being deprecated → kept as an optional strict/perf adaptor). JS execution is the core implementation.

## Hub port
- It is a **refactor, not a copy.** Shell is the target UX but built on a prior API with YAGNI. Per surface, two questions: (1) do we need it? (2) is it on the real SDK or a dead API? Drop YAGNI. **Depth matters** — sub-routes that *do* things, not just list views.
- UX is the **design team's** to own (the apps extraction enables their cloud workspace to mirror the repo). Don't guess UX. The shell home's **dock** was a preferred design worth bringing back — but the home design is the team's call.
