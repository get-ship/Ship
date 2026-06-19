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
