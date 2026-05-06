# `.sui` Dev Loop

What happens between you saving a `.sui` file and the browser reflecting the change.

## End-to-end path

```
        ┌──────────────────────────────────────────────────────────────┐
        │  edit  views/counter/page.sui  (in your editor)              │
        └───────────────────────────┬──────────────────────────────────┘
                                    │ fs notify (node:fs.watch, recursive)
                                    ▼
   ┌────────────────────────────────────────────────────────────────┐
   │  bundler.ts worker (supervised by shipd)                       │
   │    • 150 ms debounce fires                                     │
   │    • Bun.plugin() intercepts .sui → Bun.spawn(['sui-compiler'])│
   │    • Bun.build() re-bundles the dirty target                   │
   │    • emits: { kind:"module_changed", target, out_path, bytes } │
   └───────────────────────────┬────────────────────────────────────┘
                               │ JSON line on stdout
                               ▼
   ┌────────────────────────────────────────────────────────────────┐
   │  shipd (Rust)                                                  │
   │    • bundler supervisor reads module_changed event             │
   │    • calls publish_bundle_event() → BundleHmrChannel SSE fanout│
   │    • broadcasts to GET /_sui/dev/hmr                           │
   └───────────────────────────┬────────────────────────────────────┘
                               │ SSE event
                               ▼
   ┌────────────────────────────────────────────────────────────────┐
   │  Browser (EventSource subscribed to /_sui/dev/hmr)             │
   │    • subscribeHmr() in @ship/ui-runtime/dev receives event     │
   │    • dynamic import of new bundle URL (cache-busted with ?v=…) │
   │    • SUI runtime accepts updated module; DOM rebinds in place  │
   └────────────────────────────────────────────────────────────────┘
```

Typical latency from save to browser update: **< 200 ms** for a typical view (debounce 150 ms + compile + SSE push).

## What changed from the Vite era

The previous pipeline ran Vite as a separate dev server alongside `shipd`. It handled `.sui` transforms via `@ship/vite-plugin-sui`, pushed module updates over a WebSocket, and required a proxy layer between the daemon and the browser. That stack is gone:

- `packages/vite-plugin-sui/` — deleted
- `scripts/build-bundles-vite.mjs` — deleted
- `vite.config.ts` (root) — deleted
- `crates/ship-ui-framework/src/dev_assets.rs` — deleted (Step 7)

The new pipeline is: daemon starts worker, worker watches files, daemon fans out events via the existing SSE infrastructure, browser subscribes with `subscribeHmr()`.

## Browser integration

```ts
import { subscribeHmr } from '@ship/ui-runtime/dev';

const handle = subscribeHmr({
  onChanged(event) { /* event.target, event.url */ },
  onError(event) { /* event.target, event.message */ },
});

// later:
handle.close();
```

`subscribeHmr()` opens an `EventSource` to `/_sui/dev/hmr`. The default `onChanged` handler performs a dynamic import with a cache-busted URL so the browser picks up the new bundle without a full reload.

## Production path

In a release build, bundles are still baked into the daemon binary via `include_str!` in `crates/ship-ui-framework`. The worker produces those bundles via:

```sh
just dev-build-all        # one-shot: bun run runtime-assets/bundler.ts
ship runtime install      # installs bun + sui-compiler + bundler.ts
```

The `include_str!` mechanism is unchanged; only the tool that generates the assets changed.

## Triggers for a full reload

Most `.sui` edits hot-update in place. A full page reload happens when:

- You add or delete a view (the bundle manifest changes).
- You edit `bundler.ts` itself — the worker must be restarted, which the supervisor handles automatically.
- The browser loses the SSE connection and reconnects after a daemon restart.

## Debugging stuck HMR

1. Check `just logs` — look for `module_changed` lines from the bundler supervisor in the daemon log.
2. Open DevTools → Network → filter by `/_sui/dev/hmr`. The SSE stream should show incoming events.
3. If the SSE stream shows events but the DOM doesn't update, check that `subscribeHmr()` is called and that the `onChanged` handler's dynamic import URL is correct.
4. If no events appear after a `.sui` edit, confirm `watch_start` was sent (the daemon logs it on startup) and that the file is inside a watched directory.

## See also

- `docs/ship-architecture/references/docs/toolchain.md` — what `~/.ship/runtime/` contains and how the worker is installed.
- `runtime-assets/bundler.ts` — worker source; `startWatching()` is the HMR entry point.
- `packages/ui-runtime/src/dev/hmr.ts` — browser-side `subscribeHmr()` implementation.
- `apps/shipd/src/views/sui_hmr.rs` — SSE channel on the daemon side.
