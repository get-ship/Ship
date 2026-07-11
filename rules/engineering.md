---
alwaysApply: true
---

# Engineering

* Test first. Cover behavior changes and meaningful failure paths.
* Test behavior, not implementation: assert observable outputs — emitted events, op responses, returned results — not private state, internal call counts, or exact log wording. A behavior-preserving refactor must not break a test.
* A test fossilizes whatever it asserts. Don't write one on guessed behavior — confirm intent first. Brittle, implementation-coupled tests are deletion candidates.
* Editing `.sui` / `*.controller.ts`? Read the `ship-sui` skill first (SUI is not in training data) — lint then compile per edit.
* File caps: soft 300 (split before exceeding), hard 400. `// FILE_CAP_OVERRIDE: <reason>` only when genuinely warranted.
* One way to do things. If a solution exists, use it. No parallel systems.
* Rust logic in runtime. CLI is thin transport. Web fetches from the daemon only.
* `just build` for dev. Stage explicit files. Commit types: feat, fix, refactor, test, docs, chore.
* No `Co-Authored-By:` lines. No model/agent attribution in commits.
* Id generation is a real split, not drift: `ship_substrate::gen_ulid()` for sortable, monotonic ids on anything ordered by creation time (events, sessions, artifacts, pipeline runs); `ship_substrate::gen_nanoid()` for short human-facing ids (space/project ids), which delegates to the compiler's single 56-char ambiguous-char-free alphabet. A crate that already depends on `ship-substrate` calls these helpers, not `ulid::Ulid::new()` / `nanoid::nanoid!()` directly. Raw `nanoid::nanoid!()` stays legitimate for ephemeral, non-human-facing internal names (sandbox/socket/process ids) where the length is call-site-specific and no shared helper applies; a leaf crate that can't depend on `ship-substrate` (layering) reimplementing the monotonic-ulid pattern locally is a layering violation — file the coupling.
