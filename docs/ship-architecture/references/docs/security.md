---
group: Architecture
title: Security
description: Spaces, sandboxed code, the SUI renderable-execution sandbox, and per-actor permissions — the four layers Ship stacks to keep an agent from running away.
order: 99
---

A Ship agent runs as a supervised actor with a scope, a tool gate,
and an audit trail. There is no implicit "trust the agent" — every
tool call, every file read, every external integration is checked
against a policy the runtime enforces. The marketing pitch calls
this "the four layers": spaces, sandboxed code, the SUI
renderable-execution sandbox (SES on SUI), and permissions + gates.
This page is the long form.

## Layer 1 — Spaces (context, not a boundary)

A **space** is a context with its own identity, agents, and data.
Switching spaces switches what the agent can see; it does not
itself grant or revoke permissions. The actual trust boundary is
the **actor** (per-actor `cwd`, `space_id`, permissions, transport,
MCP tool gate) — not the space. Spaces are how you give an actor's
scope a name and a window.

Spaces still matter for security, but the framing is right: they
make it easy to *not* share state between contexts. The home space
has your drafts; the customer-data space is read-only. Two agents
in two spaces never see each other's tools unless they
intentionally connect. Spaces are how you avoid the cross-talk,
not how you enforce the boundary.

Read [Spaces](../concepts/spaces/) for the full mental model.

## Layer 2 — Sandboxed code (the deck webview)

Every Ship view runs in a webview the deck layer hosts. The
webview is single-origin (`localhost`, with a per-session
pid-bound token the daemon mints over the UDS host socket). The
view's compiled JS lives in this webview; the daemon serves the
shipped bundles and the SUI primitive pack from a content-hash
cache. The view never makes a `fetch` call to anything but the
daemon on `localhost`; the runtime drops raw `fetch` from view
code at compile time (see [SUI rules](../../sui/template/) §
controller purity).

Code an agent writes lands in a view, the view ships, the user
runs it. The view is the unit of code that an agent can ship to a
human reviewer. A view that talks to a service the user didn't
expect is a view the user didn't accept.

This is what "sandboxed code" means in the marketing copy: every
piece of code the agent produces is a view, every view is a
reviewable artifact, and the view can't reach outside its own
session without an explicit gate.

## Layer 3 — SES on SUI (the renderable-execution sandbox)

The SUI template compiler enforces a **renderable expression
subset** on every interpolation in a `.sui` file. Method calls are
allowlisted (`slice`, `map`, `filter`, `includes`, `toString`,
`length`, `keys`, `values`, `entries`, `has`, `get`, `toFixed`).
Arbitrary function calls, `new`, `await`, `this`, regex literals,
and tagged templates are rejected. The runtime emits compiled ESM
that hydrates against the ship SDK client — there is no
parser at runtime, and the compiled module's reach is exactly
the calls the compiler emitted.

In practice, this means a view can read from the signals and
SDK calls declared in its scope, and nothing else. A view that
says `fetch('https://attacker')` is a compile error. A view that
says `window.eval` is a compile error. A view that opens a
sensitive file is a runtime error — the SDK call is gated by the
actor's tool gate, not by the view.

The full subset rule is documented in the SUI compiler source
under `/home/madvib/dev/sui/crates/sui/compiler/src/check/expr_subset.rs`. It is the
single piece of code that decides "can this view call this?". The
rule is small on purpose — every line in the allowlist is one
the maintainers have read.

## Layer 4 — Permissions + gates (the actor's tool gate)

Every agent profile declares a permission preset
(`ship-readonly`, `ship-standard`, `ship-autonomous`, or
`ship-elevated`) and may layer additional rules. The actor's
tool gate reads the preset at spawn and rejects any tool call
that isn't in the resolved allow-list (or, for `ask` rules, that
the user hasn't approved). The gate is the runtime's contract
with the user: an agent that says it's read-only cannot edit a
file, full stop, regardless of what its prompt says.

The four presets are layered, not exclusive. `ship-readonly`
is a strict subset of `ship-standard`; `ship-elevated` is
`ship-autonomous` plus `git push` and publish. Custom presets
live in `.ship/permissions.jsonc` and reference the same
allow/deny/ask grammar. Read
[Permissions](../../agents/permissions/) for the full surface.

A gate is per-actor, not per-space. Two agents in the same space
with different presets have different gates. The actor row
carries the resolved permission set; the supervisor re-evaluates
it on every tool call.

## What the four layers do together

The layers compose. A request from an agent to do something
risky — read a secret file, push to a protected branch, fetch a
URL — has to clear:

1. **Permission gate**: is this tool allowed for this actor at all?
2. **Tool gate**: is this specific call (with this specific arg)
   in the resolved allow-list?
3. **View scope (if the call originates in a view)**: is the
   underlying SDK call even exposed to this view's scope?
4. **SUI subset (if the call originates in a template)**: is the
   expression shape even in the renderable subset?
5. **Audit (always)**: the call lands on the event log; the user
   can scrub back, replay, and revoke.

A failure at any one layer is a hard stop. A success at all five
is a real tool call. The layers are not redundant — they catch
different failure modes. The permission gate catches a prompt
injection that asks the agent to do something out of scope. The
tool gate catches a misconfigured preset. The view scope catches
a view that wasn't expecting the call. The SUI subset catches a
view that was authored by an attacker (or by an agent in a
compromise). The audit catches everything else, after the fact.

## What the layers do *not* do

- **They don't make Ship a sandbox.** Ship runs your code. The
  SUI subset and the tool gate keep your code from doing
  surprising things; they don't keep your code from doing what
  you told it to do.
- **They don't replace review.** A `ship-elevated` agent can push
  to main if its preset allows it. A human reviewer gating
  dangerous merges is a workflow choice, not a runtime guarantee.
- **They don't hide secrets from the user.** Ship's secret store
  (planned) encrypts values the user added; it doesn't keep
  values the user put in a file from being read by an agent the
  user told to read files.

## Where to go next

- [Spaces](../concepts/spaces/) — the context unit, in depth.
- [Permissions](../../agents/permissions/) — the four presets,
  the layering rules, custom presets.
- [Runtime Model](../runtime-model/) — the storage backends, the
  four primitives, the decision rules for where security state
  lives.
