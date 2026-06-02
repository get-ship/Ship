---
name: ship-docs
stable-id: ship-docs
description: The Ship documentation convention. Use before writing any doc — decides WHERE a fact lives (code comment, per-crate README, ADR, memento, or nowhere) so we never grow a SKILL.md per module again. Covers comment density and what a per-crate README holds.
tags: [docs, convention, readme, adr]
authors: [ship]
audience: internal
---

# How we document

One rule: **document the thing next to the thing.** A fact lives in exactly
one place — the place closest to the code it describes. Anything that mirrors
code drifts; put it where the compiler keeps it honest.

## Where a fact goes

Pick the first row that fits. Do not duplicate across rows.

| The fact is... | It lives in... |
|---|---|
| how a function/type behaves, an invariant, a gotcha at a call site | a **code comment** (`///` doc comment on the item, `//` inline for the gotcha) |
| what a crate is, its public surface, how its modules fit, how to run it | the **crate README** (`crates/.../<crate>/README.md`) |
| a decision that constrains future work (a boundary, a trade-off, a "we will not do X") | an **ADR** (`docs/ship-adrs/` — context, decision, alternatives, consequences) |
| a correction, operational rule, mistake, or rejected idea you don't want relitigated | **memento** (experimental) — `.ship/skills/memento/` |
| reference material for humans + researching agents (the docs site) | a **doc-skill page** under `docs/ship-*/references/docs/` — see `writing-docs` |
| it's already obvious from the code, names, and types | **nowhere.** Delete the comment. |

If two rows seem to fit, the higher one wins: prefer the comment over the
README, the README over a doc page. The lowest-ceremony home that keeps the
fact honest is the right one.

## No SKILL.md per module

A per-module `SKILL.md` that narrates code is an anti-pattern — it is a second
copy of the truth that the build never checks, so it rots. **Do not create
one.** Module/crate truth belongs in the crate README + doc comments. Skills
are for *operational instructions an agent follows* (how to author a view, how
to run a gate), not for describing what a crate contains.

## Comment density

Comment intent and constraints, not mechanics.

- **Do** write a `//!` module header (one paragraph: what this module owns) and
  `///` on every public item explaining *why it exists* and any invariant a
  caller must respect.
- **Do** flag the non-obvious: a deliberate ordering, a footgun, a boundary
  ("L2 has no privileged access — consume the SDK").
- **Don't** restate the code (`// increment i`). If the line needs a comment to
  be read, rename until it doesn't, then keep the comment only for the *why*.

## What a per-crate README holds

Keep it short — a map, not a manual. Six sections, drop any that are empty:

1. **One line** — what this crate is, in the four-layer model (L0/L1/L2/L3).
2. **Responsibilities** — the 3–6 things it owns. Bullet list.
3. **Public surface** — the key types/traits a consumer touches, one line each.
4. **Boundaries** — what it must NOT do, who it may/may not depend on.
5. **Run/test** — the exact commands (`cargo test -p <crate>`, fixtures, env).
6. **Pointers** — links to the ADR(s) and doc-site pages that go deeper.

Anything longer than a screen is a sign a fact wants to be an ADR or a doc page
instead.

## README vs ADR vs memento vs doc-page vs nothing

- **README** answers "what is this and how do I use it." Present tense, current
  state. Update it when the crate changes.
- **ADR** answers "why is it this way, and what did we reject." Immutable record
  of a decision with constraints; supersede, don't edit.
- **memento** is the running log of corrections/rules so agents stop
  relitigating — experimental, local, not a product surface.
- **doc-page** (`references/docs/`) is published reference for humans and
  researching agents; it feeds the docs site.
- **nothing** is the most common correct answer. If the code says it clearly,
  adding prose adds drift surface. Write less.
