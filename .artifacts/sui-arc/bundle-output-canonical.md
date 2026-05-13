# bundle-output-canonical

Canonical output paths for SUI bundles under `crates/ship-sui-server/assets/`:

- `assets/<name>.js` — always-loaded framework bundles
  (`sui-runtime.js`, `sui-dom.js`, `sui-sdk-client.js`, `sui-components.js`)
  and out-of-band assets (`lucide-icons.json`, `sui-custom-elements.json`).
- `assets/sui-islands/<name>.js` — every code-split island bundle (the
  set enumerated in `assets.rs::is_known_island`).
- `assets/views/<name>/view.js` — per-view bundles.

## What the two writers do

- `runtime-assets/bundler.ts` (in-daemon Bun worker): discovers islands
  under `packages/ui-components/src/islands/` and writes
  `sui-islands/<name>.js` directly. Framework bundles + views land at
  the same canonical locations.
- `packages/sui-build` driven by `packages/ui-components/sui.config.ts`:
  always-loaded `entries` write to `outdir/<name>.js`, code-split
  `lazyBundles` write to `outdir/<lazyOutSubdir>/<name>.js`. The
  ui-components config sets `lazyOutSubdir: 'sui-islands'` so both
  writers land identical files at identical paths.

## Why this layout

Before this job both writers targeted different paths and
`crates/ship-sui-server/src/assets.rs` fell back through both at request
time. A partial rebuild left stale bytes behind, since whichever writer
ran last wrote one path and the daemon could still serve the other.
Collapsing to one canonical path eliminates the split brain.

## Acceptance

- `find crates/ship-sui-server/assets -name "sui-*.js"` lists exactly
  one copy of each island (the one under `sui-islands/`).
- `assets.rs`'s `include_str!` paths and `serve_sui_island` disk read
  point only at `assets/sui-islands/<bundle>.js`. No legacy fallback.
- `cargo build -p ship-sui-server` and the sui-build vitest suite
  succeed.
