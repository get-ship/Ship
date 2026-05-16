# Job Spec: Docs site uses SUI (not Lit) — showcase the framework it documents

**Branch:** job/docs-site-sui
**Agent:** ship-designer
**Mode:** autonomous

## Goal

The docs site at `apps/docs/` (Astro + Markdoc) is the public surface where users learn what Ship is. Today it likely uses Lit components from `packages/ship-set/` for any interactive content. With j2/lit on the deletion path, the docs site needs to **use SUI components and showcase what SUI can do** — both because we're killing the Lit dep tree, and because *the docs site demonstrating the framework it documents* is the right shape.

This isn't a content rewrite. Existing Markdoc content stays. The work is replacing any Lit/ship-set component usage with SUI equivalents, and adding interactive SUI demos where they sharpen what the docs are explaining.

## What to do

1. **Inventory current state.** Read `apps/docs/`:
   - `package.json` — what's it depending on? Look for `@ship/set`, `lit`, anything j2-shaped.
   - Astro config and components — what's actually being used as interactive content?
   - Markdoc nodes/tags — what custom components are registered?
2. **Identify Lit / ship-set usage.** Every import of `@ship/set` or `lit`. Every place a Lit component is rendered in the site. Catalog them.
3. **Replace with SUI equivalents.** For each Lit usage, swap to the SUI primitive (use the SUI Radix-parity work as the source of truth for what's available — that job runs in parallel; coordinate via the parity matrix at `.artifacts/launch-arc/reports/sui-radix-parity.md` if it exists). If a needed SUI primitive doesn't exist yet, log it via `mcp__ship__log_progress` — don't port a Lit component just for the docs site.
4. **Add interactive SUI demos** in 2-3 high-value places (component pages, getting-started page, key feature pages). The goal: visitors see SUI in action on the docs site itself, not just descriptions of SUI. Don't go overboard — 2-3 demos that sharpen specific content sections are better than retrofitting everything.
5. **Remove Lit/ship-set deps** from `apps/docs/package.json` once they're unused. `bun install` to verify.
6. **Verify the site builds and serves.** `bun run --filter @ship/docs build` (or whatever the actual command is in `apps/docs/package.json`). Run the site locally and confirm pages render correctly.

## File scope

`apps/docs/` (all subdirectories), `apps/docs/package.json` (dep removal). May add new SUI demo components in `apps/docs/src/components/` if needed for demo embedding. Do NOT touch `packages/ship-set/` (it's slated for deletion separately). Do NOT touch the marketing site at `apps/site/` (separate job).

Do NOT add the `.artifacts/` line to `.gitignore`.

## Architectural context
- `apps/docs/` is Astro 4+ with Markdoc. Astro can render web components directly (SUI tags work in `.astro` and `.md` files).
- SUI components are registered globally when the runtime loads — the docs site needs to load `@ship/sui-runtime` and the relevant island bundles. Check whether the site currently has this wiring or only had Lit.
- Markdoc custom tags are defined in `apps/docs/markdoc.config.mjs`. Adding interactive demos means registering tags that render SUI components.
- The marketing site at `apps/site/` is a separate concern — don't touch it.
- CLAUDE.md: no scaffolds. If a SUI primitive needed for the docs site doesn't exist, log the gap and use a static fallback — don't half-build the primitive.

## Acceptance criteria

1. `apps/docs/package.json` has no `@ship/set`, `lit`, or `lit-*` dependencies.
2. No `import` of `@ship/set` or `lit` anywhere in `apps/docs/src/`.
3. Any interactive content in the docs site renders via SUI components (using `<sui-*>` tags directly or as wrapped Astro components).
4. At least 2 interactive SUI demos added at meaningful content locations — list the locations in the commit message.
5. `bun run --filter @ship/docs build` succeeds. The built site serves and pages render without console errors.
6. `git diff --stat` confined to `apps/docs/` + any new SUI demo components.

---
> Execute this job yourself. Do not delegate the implementation to a subagent — subagents are only for read-only parallel research, never for writing the code this job asks for.
>
> If you notice a bug or problem outside your file scope (e.g. a missing SUI primitive, a Lit component without a SUI counterpart), log it via `mcp__ship__log_progress` and continue your work.
>
> **When all acceptance criteria are met:**
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary of what changed.
> 2. Call `mcp__ship__mesh_broadcast` with `{ "topic": "job.completed", "workspace_id": "<id>", "summary": "<same summary>" }`. If shipd is not running, skip step 2.
