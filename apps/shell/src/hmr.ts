// Dev-only HMR glue between `@ship/sui-runtime/dev`'s subscribeHmr and
// shell's mount lifecycle.
//
// Bundler target shape (see packages/deck/bundler/index.ts):
//   views/<name>/view      → one bundle per views/<name>/page.sui
//   sui-components/<name>  → separate-bundle component
//   sui-runtime, sui-dom, sui-components, ship-elements, …
//
// Behavior:
//   - views/<name>/view AND <name> === currentRoute.view
//     → cache-bust import the rebuilt view.js, re-mount in place.
//   - any other target (components, islands, runtime)
//     → full page reload (custom-element re-registration without reload
//       is out of scope for v1).
//   - compile errors → log to console with file:line.
//
// No-op in production builds: Vite inlines `import.meta.env.DEV` to false,
// the dead-code branch is dropped, and the EventSource is never opened.

import { subscribeHmr } from '@ship/sui-runtime/dev'

interface MountModule {
  default: { setup: (...args: unknown[]) => unknown }
}

export interface HmrDeps {
  getCurrentView(): string | null
  remount(module?: MountModule): Promise<void>
}

export function initHmr(deps: HmrDeps): { close(): void } | null {
  if (!isDev()) return null
  if (typeof window === 'undefined' || typeof EventSource === 'undefined') return null

  const sub = subscribeHmr({
    onChanged: async (event) => {
      const target = event.target
      if (target.startsWith('views/')) {
        const segs = target.split('/')
        const viewName = segs[1]
        const current = deps.getCurrentView()
        if (!viewName || current !== viewName) {
          // A different view's bundle changed; nothing to do until the
          // user navigates to it (next mount will fetch the new payload).
          return
        }
        const url = `/_sui/${target}.js?v=${event.bytes}`
        try {
          const mod = (await import(/* @vite-ignore */ url)) as MountModule
          await deps.remount(mod)
          console.info(`[sui:hmr] re-mounted ${viewName}`)
        } catch (err) {
          console.error(`[sui:hmr] re-mount of ${viewName} failed; reloading`, err)
          window.location.reload()
        }
        return
      }
      // Component / runtime / separate-bundle changed — full reload.
      console.info(`[sui:hmr] ${target} changed; reload`)
      window.location.reload()
    },
    onError: (err) => {
      const loc = err.file ? ` (${err.file}${err.line ? `:${err.line}` : ''})` : ''
      console.error(`[sui:hmr] compile error in ${err.target}${loc}:`, err.message)
    },
  })

  const onUnload = () => sub.close()
  window.addEventListener('beforeunload', onUnload)
  return sub
}

function isDev(): boolean {
  // Vite's define-replace plugin rewrites the exact string
  // `import.meta.env.DEV` to a literal `true` / `false`. Aliasing
  // through a local (`const m = import.meta; m.env?.DEV`) defeats the
  // string match, leaving `env` undefined at runtime — so the previous
  // form silently never enabled HMR under Vite. Read the property
  // directly. `(import.meta as Any).env.DEV` is rewritten by Vite
  // because the literal `import.meta.env.DEV` substring is present in
  // the cast expression's text. Cast because shell's tsconfig doesn't
  // include `vite/client` types.
  type V = ImportMeta & { env: { DEV: boolean } }
  if ((import.meta as V).env.DEV === true) return true
  const w = window as Window & { __SHIP_DAEMON_MODE?: string }
  return w.__SHIP_DAEMON_MODE === 'development'
}
