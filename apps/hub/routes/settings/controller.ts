// <sui-settings> — CLI + studio configuration. Hub port of
// `apps/shell/routes/settings/controller.ts`, rendered at /settings.
//
// Read shape (#298): the daemon's ONLY settings read surface is the flat
// `SettingsService` (proto/ship/ops/v1/settings.proto) — RUNTIME-scope rows
// merged with hardcoded defaults. This controller composes the template's scope
// from the flat rows: unary `ops.settings.list(RUNTIME)` supplies the values;
// `ops.settings.watch(RUNTIME)` is a render trigger that re-reads the list on
// every settings.* event. Writes go through unary `ops.settings.set`.
// `ship.account.state` stays LIVE — it is an on-contract mux resolver.
//
// Client seam: `ship` + `ops` are AMBIENT — pulled off `ctx.clients.<name>`
// (the host's entry.gen.ts constructs them from src/lib/client.ts and the
// deck-runtime maps `clients.<name>` into view scope). The shell's
// `../../ops` import (module-level `ops`/`ship`/`writeSetting`) is gone; the
// settings source + write helper take the ambient `ops` directly.
import { controller, signal, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { LiveSignal, LiveSource, Signal, ViewScope } from '@ship/sui-runtime'
import type { AccountState } from '@ship/sdk-client'
import {
  accountDisplayName,
  accountInitials,
  accountPlanLabel,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
} from '@ship/sdk-client'
import { SettingScope } from '@ship/sdk/connect'
import type { ShipConnectClient } from '@ship/sdk/connect'
import type { Scope } from './page.sui'

/** Flat RUNTIME settings rows (stored merged with defaults), keyed by setting key. */
type SettingsMap = Record<string, unknown>

/**
 * LiveSource over the flat RUNTIME settings table. `Set` writes author a
 * `settings.changed` kernel event, the Watch stream emits a frame, and the
 * source re-reads the list — so a save made anywhere (this page, the CLI,
 * another tab) lands back in the template. Takes the ambient Connect client.
 */
function runtimeSettingsSource(ops: ShipConnectClient): LiveSource<SettingsMap> {
  return {
    subscribe(sig) {
      const ctrl = new AbortController()
      const dec = new TextDecoder()
      const refresh = async (): Promise<void> => {
        const list = await ops.settings.list({ scope: SettingScope.RUNTIME })
        const map: SettingsMap = {}
        for (const s of list.settings) {
          if (s.valueJson.length === 0) continue
          try {
            map[s.key] = JSON.parse(dec.decode(s.valueJson)) as unknown
          } catch {
            // One malformed row never blocks the rest of the page.
          }
        }
        sig.set({ kind: 'ok', value: map })
      }
      void (async () => {
        try {
          await refresh()
          for await (const _frame of ops.settings.watch(
            { scope: SettingScope.RUNTIME },
            { signal: ctrl.signal },
          )) {
            await refresh()
          }
        } catch (err) {
          if (!ctrl.signal.aborted) {
            sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
          }
        }
      })()
      return () => ctrl.abort()
    },
  }
}

function flashSaved(sig: Signal<boolean>, ms = 1600): void {
  sig.set(true)
  const w = window as unknown as Record<string, number | undefined>
  const key = `__shipSetSavedTimer_${(sig as unknown as { __id?: number }).__id ?? Math.random()}`
  const existing = w[key]
  if (typeof existing === 'number') clearTimeout(existing)
  w[key] = setTimeout(() => sig.set(false), ms) as unknown as number
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.clients.ops
    const ship = ctx.clients.ship
    // Live sources: the flat RUNTIME settings table (List + Watch) and the
    // on-contract `account.state` live resolver.
    const settings = live(() => runtimeSettingsSource(ops))
    const account = live(() => ship.account.state())

    // `<form mutation="ship.settings.update">` resolves to this closure (returned
    // as `ship.settings.update` below). It writes one RUNTIME setting as JSON
    // bytes via the unary `ops.settings.set`.
    async function settingsUpdate(formData: FormData): Promise<{ key: string; value: string }> {
      const key = String(formData.get('key') ?? '')
      const value = String(formData.get('value') ?? '')
      if (!key) throw new Error('settings.update: missing "key" field')
      await ops.settings.set({
        key,
        valueJson: new TextEncoder().encode(JSON.stringify(value)),
        scope: SettingScope.RUNTIME,
      })
      return { key, value }
    }

    // Per-key string reads over the flat map. Values are arbitrary JSON
    // (defaults arrive as strings, stored rows may be numbers/bools) —
    // inputs always bind a string.
    const empty: SettingsMap = {}
    const settingStr = (key: string): string => {
      const v = unwrapOr(settings, empty)[key]
      return v == null ? '' : typeof v === 'string' ? v : String(v)
    }

    const accountBusy = signal(false)
    const accountError = signal('')
    const accountIdentity = derived(() => readAccountIdentity(unwrapOr(account as LiveSignal<AccountState | null>, null)))
    const accountName = derived(() => accountDisplayName(accountIdentity()))
    const accountInitialsSig = derived(() => accountInitials(accountIdentity()) || '··')
    const accountPlan = derived(() => accountPlanLabel(accountIdentity()))

    async function onAccountSignIn(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true); accountError.set('')
      try { await accountSignIn() }
      catch (e) { accountError.set(e instanceof Error ? e.message : String(e)) }
      finally { accountBusy.set(false) }
    }
    async function onAccountSignOut(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true); accountError.set('')
      try { await accountSignOut() }
      catch (e) { accountError.set(e instanceof Error ? e.message : String(e)) }
      finally { accountBusy.set(false) }
    }

    return {
      settings, account,
      // One derived per rendered setting — the template binds these, never
      // a nested shape the daemon doesn't serve.
      terminalProgram: derived(() => settingStr('terminal.program')),
      worktreeBase: derived(() => settingStr('worktree.base_dir')),
      harnessDefault: derived(() => settingStr('harness.default')),
      harnessDefaultModel: derived(() => settingStr('harness.default_model')),
      autoResume: derived(() => {
        const v = unwrapOr(settings, empty)['agent_session.auto_resume']
        return v === true || v === 'true'
      }),
      registryEndpoint: derived(() => settingStr('registry.endpoint')),
      watcherDebounceMs: derived(() => settingStr('watcher.debounce_ms')),
      gitLogLimit: derived(() => settingStr('git.log_limit')),
      hooksMaxDepth: derived(() => settingStr('hooks.max_depth')),
      // The origin this page is served from IS the daemon endpoint (same-origin
      // Connect transport, src/lib/client.ts).
      daemonEndpoint: derived(() =>
        typeof window === 'undefined' ? '' : window.location.origin),
      advancedOpen: signal(false),
      savedTerminal: signal(false),
      savedWorktree: signal(false),
      savedRegistry: signal(false),
      savedHarnessDefault: signal(false),
      savedHarnessModel: signal(false),
      savedAutoResume: signal(false),
      savedWatcherDebounce: signal(false),
      savedGitLogLimit: signal(false),
      savedHooksMaxDepth: signal(false),
      flashSaved,
      ship: { settings: { update: settingsUpdate } },
      accountBusy, accountError, accountIdentity, accountName,
      accountInitials: accountInitialsSig, accountPlan,
      onAccountSignIn, onAccountSignOut,
      openBilling: openAccountBilling,
    }
  },
  on: {
    rerunSetup: () => {
      try {
        localStorage.removeItem('ship-setup-completed')
        localStorage.removeItem('ship-setup-state')
      } catch { /* best-effort */ }
      window.dispatchEvent(new CustomEvent('ship:setup:rerun'))
      if (!window.location.pathname.startsWith('/setup')) {
        window.location.href = '/setup'
      }
    },
  },
})
