# Engineering

* Test first. Cover behavior changes and meaningful failure paths.
* File cap: 300 lines. Split before exceeding.
* One way to do things. If a solution exists, use it. No parallel systems.
* Rust logic in runtime. CLI/MCP are thin transport. Web fetches from the daemon only.
* `just build` for dev. Stage explicit files. Commit types: feat, fix, refactor, test, docs, chore.
* No `Co-Authored-By:` lines. No model/agent attribution in commits.
