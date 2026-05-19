# Job Spec: retire `/theme-studio` and `/ship-ui-studio` legacy routes

**Branch:** `job/old-routes-cleanup`
**Agent:** ship-designer
**Mode:** autonomous
**Conditional on:** `job/themes-domain` AND `job/components-domain` having merged. DO NOT dispatch until both their gates have passed.

## Goal

After `themes-domain` and `components-domain` land the new `/themes` + `/components` routes, the old `/theme-studio` and `/ship-ui-studio` views remain alive for backward compat. This job retires them.

## File scope

```
views/theme-studio/               # remove entirely (after redirect)
views/ship-ui-studio/             # remove entirely (after redirect)
[router config]                   # add 301 redirects (path TBD by current routing setup)
[shell rail / nav]                # remove any links pointing to old routes
[docs / READMEs]                  # update path references
```

## What to change

1. **Add 301 redirects**: `/theme-studio` → `/themes`, `/theme-studio/<id>` → `/themes/<id>` (if applicable). Same for `/ship-ui-studio` → `/components`.
2. **Remove rail entries / nav links** referencing the old paths (shell-chrome refactor likely removed the rail entirely, but search for any leftover hardcoded refs).
3. **Sweep docs and READMEs** for old route references — update or remove.
4. **Delete `views/theme-studio/` and `views/ship-ui-studio/` directories**.
5. **Verify**: production paths `bun packages/deck/bundler/index.ts --all` succeeds. No broken imports. No tests reference the old routes.

## Acceptance criteria

1. `views/theme-studio/` and `views/ship-ui-studio/` directories deleted.
2. `git grep theme-studio` returns only redirect / migration references (or zero).
3. `git grep ship-ui-studio` same.
4. Old paths return 301 redirects in the running daemon (or whatever the equivalent is in your router — at minimum the user navigating to an old path lands on the new one).
5. `bun run build` passes everywhere. Bundler succeeds.
6. Catalog / catalog tests pick up the new routes correctly.

## Out of scope

- Re-routing `/agents`, `/skills` (those are still canonical paths — only the two "studio" routes are retired in this job).
- Studio absorption of `/agents`, `/skills`, etc. into `/studio/*` — separate later job.

---

> CONDITIONAL ON wave-2 (themes-domain + components-domain) merging. Do not dispatch until.
>
> When done: complete_workspace + mesh_broadcast.
