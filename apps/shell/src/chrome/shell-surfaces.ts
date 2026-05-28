// Hub-internal routable surfaces — `.sui` trees BUNDLED INTO THE HUB
// (as `sui-components/<name>.js`) that mount from the hub bundle rather
// than resolving as daemon-served views. The hub itself is daemon-served
// at `/hub` (#171), but these surfaces are still hub-bundled — the
// daemon has no `view-source/<name>` for them, so anything that fetches
// a view's daemon source (devtools source/schema tabs) must skip them
// or every visit logs a spurious 404.
//
// Keep in sync with the local-mount gate in `main.ts` (`mountRoute`) and the
// shell build (`deck build`, crates/deck/deck-cli). Add a name here when a
// new hub-internal surface lands.
//
// `studio` is daemon-served from `apps/shell/views/` (the daemon view-source
// endpoint resolves `views/` only, so it still 404s); the rest are chrome
// components (`apps/shell/src/components/sui-*`) rendered directly — no daemon
// source at all. Either way the devtools source/schema fetch must skip them.
// Retiring this gate requires restructuring those chrome components as
// daemon-served views; out of scope for the #171 hub-flip.
const SHELL_LOCAL_VIEWS: ReadonlySet<string> = new Set([
  'studio', 'home', 'activity', 'account', 'library', 'setup', 'settings', 'canvas',
])

/** True when `name` is a shell-owned surface with no daemon view source. */
export function isShellLocalView(name: string): boolean {
  return SHELL_LOCAL_VIEWS.has(name)
}
