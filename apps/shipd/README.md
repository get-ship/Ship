# shipd — Ship daemon

HTTP API + hypermedia view server. Serves the shell, canvas, pipeline runs,
and the `<ship-*>` web-component bundles that views import.

## Asset bundles (built on demand, not tracked)

The daemon embeds four JS bundles via `include_str!`:

- `assets/ship-elements.js`
- `assets/ship-code-editor.js`
- `assets/ship-markdown-editor.js`
- `assets/ship-terminal.js`

These are **not committed** — they are built from source in
`packages/ui/src/web-components/` by the top-level script:

```bash
pnpm build:bundles           # build all (idempotent; skips unchanged)
pnpm build:bundles -- --force    # rebuild everything
pnpm build:bundles:watch     # rebuild on source change
```

Run this **before** `cargo build -p shipd`, `cargo test -p shipd`, or
`cargo run -p shipd`. CI runs it in the Rust job. The bundles (and the
`.cache/` directory used for hash-skip) are gitignored.

`assets/datastar.js` and `assets/view-tokens.css` *are* tracked — the first
is a 30 KB upstream client, the second is authored by us.
