// Shell-owned routable surfaces — `.sui` trees that SHIP WITH THE SHELL
// (bundled as `sui-components/<name>.js`) and mount from the local bundle
// rather than resolving as daemon-served views. The daemon has no
// `/api/v1/view-source/<name>` or `/api/v1/views/<name>/files` for these,
// so anything that fetches a view's daemon source (devtools source/schema
// tabs) must skip them — otherwise every visit logs a spurious 404.
//
// Keep in sync with the local-mount gate in `main.ts` (`mountRoute`) and
// the bundler's `SHELL_SURFACES` (packages/deck/bundler/index.ts). There is
// one today; add a name here when a new shell-owned surface lands.
const SHELL_LOCAL_VIEWS: ReadonlySet<string> = new Set(['studio'])

/** True when `name` is a shell-owned surface with no daemon view source. */
export function isShellLocalView(name: string): boolean {
  return SHELL_LOCAL_VIEWS.has(name)
}
