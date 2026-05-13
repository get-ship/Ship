// Dev-only HMR glue between `@ship/sui-runtime/dev`'s subscribeHmr and
// shell2's mount lifecycle.
//
// Bundler target shape (see runtime-assets/bundler.ts):
//   views/<name>/view   → one bundle per views/<name>/page.sui
//   sui-islands/<name>  → component island bundle
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
      // Component / runtime / island bundle changed — full reload.
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
  // Vite inlines `import.meta.env.DEV` as a literal at build time. Cast
  // because shell2's tsconfig doesn't include `vite/client` types.
  const meta = import.meta as ImportMeta & { env?: { DEV?: boolean } }
  if (meta.env?.DEV === true) return true
  const w = window as Window & { __SHIP_DAEMON_MODE?: string }
  return w.__SHIP_DAEMON_MODE === 'development'
}
