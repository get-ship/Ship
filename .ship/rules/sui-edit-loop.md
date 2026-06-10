# SUI Edit Loop

After every edit to a `.sui` or `*.controller.ts` file, run lint then compile — per edit, not at PR time.

* Lint: `cargo run -p sui-lint --example check -- <file>` (exits 1 on findings).
* Compile: `cargo run -p sui-compiler -- build <file>` (full check, exits non-zero on Error).

Both must pass before the edit is done. Commands and rationale live in the `ship-sui` skill.
