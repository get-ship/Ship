<controller>
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
// Client seam: `ship` + `ops` are AMBIENT — pulled off `ctx.<name>`
// (the host's entry.gen.ts constructs them from src/lib/client.ts and the
// deck-runtime maps `clients.<name>` into view scope). The shell's
// `../../ops` import (module-level `ops`/`ship`/`writeSetting`) is gone; the
// settings source + write helper take the ambient `ops` directly.
import { controller, signal, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { LiveSignal, LiveSource, Signal, ViewScope } from '@ship/sail-runtime'
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
import type { Scope } from './page.sail'

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
    const ops = ctx.ops
    const ship = ctx.ship
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
</controller>

<!-- <sui-settings> — CLI + studio config. First-party CHROME (apps/shell); rendered at /settings.
     Composed from @ship/sail-primitives: sections are <card>, rows are
     <row>/<column>, inputs are <input>, the account avatar is
     <avatar>, the read-only endpoint is <code-block>. The co-located
     style.css carries only the residue primitives can't express; inline
     <style> blocks in a route template are dropped (FW-4). -->

<!-- Settings read shape (#298): the controller composes flat RUNTIME
     SettingsService rows into one derived per rendered setting. This template
     binds those deriveds — it never dereferences a nested settings document,
     because the daemon does not serve one. -->

<div>
  <main>
    <header>
      <h1>Settings</h1>
      <p>CLI and studio configuration</p>
    </header>

    
      <while-loading of="settings">
      <column gap="4" aria-busy="true" aria-label="Loading settings">
        <for as="n" of={[0, 1, 2]} key={n}>
          <card>
            <column gap="3">
              <row gap="2" align="center">
                <spinner shape="circle" size="sm"></spinner>
                <spinner shape="line" size="md" style="width:120px"></spinner>
              </row>
              <column gap="2">
                <spinner shape="line" size="md"></spinner>
                <spinner shape="line" size="md"></spinner>
              </column>
            </column>
          </card>
        </for>
      </column>

      </while-loading><while-error as="err" of="settings">
      <error-state variant="default" size="md" title="Couldn't load settings">
        {err.message}
      </error-state>

      </while-error><if cond={settings().kind === 'ok'}>
      <column gap="4">

        <!-- Account -->
        <card id="account" padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="user" size="sm" tone="muted"></icon>
              <h2>Account</h2>
            </row>
          </div>
          <column gap="3">
            <if cond={account().kind === 'loading'}>
              <span>Loading account…</span>
            <else-if cond={account().kind === 'error'}>
              <column gap="1">
                <span>Ship Cloud</span>
                <span>Daemon unreachable. Account features unavailable while offline.</span>
              </column>
            </else-if><else-if cond={accountIdentity().status === 'logged_in'}>
              <row gap="3" justify="between" align="center">
                <row gap="3" align="center">
                  <avatar size="sm" tone="primary"
                    src={accountIdentity().me?.image ?? ''}
                    name={accountName()}
                    initials={accountInitials()}></avatar>
                  <column gap="0">
                    <span>{accountName()}</span>
                    <span>{accountIdentity().me?.email ?? 'Signed in to Ship Cloud'}</span>
                  </column>
                </row>
                <if cond={accountPlan()}>
                  <badge tone="info" variant="soft" size="sm">{accountPlan()}</badge>
                </if>
              </row>
              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Billing</span>
                  <span>Manage your plan and payment method</span>
                </column>
                <button variant="outline" size="sm" on:click={() => openBilling()}>
                  <icon slot="leading" name="external-link" size="sm"></icon>
                  Open billing
                </button>
              </row>
              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Session</span>
                  <span>Sign out clears the API key stored by this daemon</span>
                </column>
                <button variant="outline" tone="destructive" size="sm"
                  disabled={accountBusy()}
                  on:click={onAccountSignOut}>
                  Sign out
                </button>
              </row>
            </else-if><else-if cond={accountIdentity().status === 'pending'}>
              <row gap="3" align="center">
                <spinner size="sm"></spinner>
                <column gap="1">
                  <span>Completing sign-in…</span>
                  <span>Finish the flow in your browser, then return here.</span>
                </column>
              </row>
            </else-if><else>
              <row gap="3" justify="between" align="center">
                <row gap="3" align="center">
                  <avatar size="sm" tone="muted"></avatar>
                  <column gap="1">
                    <span>Not signed in</span>
                    <span>Connect this workspace to Ship Cloud to unlock publish + sync.</span>
                  </column>
                </row>
                <button variant="solid" tone="primary" size="sm"
                  disabled={accountBusy()}
                  on:click={onAccountSignIn}>
                  Sign in
                </button>
              </row>
            </else></if>
            <if cond={accountError()}>
              <span>{accountError()}</span>
            </if>
          </column>
        </card>

        <!-- Theme -->
        <card padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="eye" size="sm" tone="muted"></icon>
              <h2>Theme</h2>
            </row>
          </div>
          <column gap="3">
            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>Customize design tokens</span>
                <span>Tweak palette, surfaces, motion, typography — overrides ride on top of the active pack.</span>
              </column>
              <a href="/theme-studio" data-test="settings-theme-studio-link">
                Open Theme Studio
              </a>
            </row>
          </column>
        </card>

        <!-- Terminal -->
        <card padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="terminal" size="sm" tone="muted"></icon>
              <h2>Terminal</h2>
            </row>
          </div>
          <column gap="3">
            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>Terminal program</span>
                <span>Used when launching sessions</span>
              </column>
              <form mutation="ship.settings.update" on:success={() => flashSaved(savedTerminal)}>
                <input type="hidden" name="key" value="terminal.program" />
                <row gap="2" align="center" justify="end">
                  <badge tone="success" variant="soft" size="sm" data-visible={savedTerminal()}>
                    <icon slot="leading" name="check" size="sm"></icon>Saved
                  </badge>
                  <input name="value" size="sm" label="Terminal program"
                    value={terminalProgram()} placeholder="tmux"
                    on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                    on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                </row>
              </form>
            </row>

            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>Worktree base directory</span>
                <span>Root for git worktrees</span>
              </column>
              <form mutation="ship.settings.update" on:success={() => flashSaved(savedWorktree)}>
                <input type="hidden" name="key" value="worktree.base_dir" />
                <row gap="2" align="center" justify="end">
                  <badge tone="success" variant="soft" size="sm" data-visible={savedWorktree()}>
                    <icon slot="leading" name="check" size="sm"></icon>Saved
                  </badge>
                  <input name="value" size="sm" label="Worktree base directory"
                    value={worktreeBase()} placeholder="/path/to/worktrees"
                    on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                    on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                </row>
              </form>
            </row>
          </column>
        </card>

        <!-- Harness defaults -->
        <card padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="cpu" size="sm" tone="muted"></icon>
              <h2>Harness defaults</h2>
            </row>
          </div>
          <column gap="3">
            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>Default harness</span>
                <span>Harness used when `ship use --launch` runs with no `--harness` flag</span>
              </column>
              <form mutation="ship.settings.update" on:success={() => flashSaved(savedHarnessDefault)}>
                <input type="hidden" name="key" value="harness.default" />
                <row gap="2" align="center" justify="end">
                  <badge tone="success" variant="soft" size="sm" data-visible={savedHarnessDefault()}>
                    <icon slot="leading" name="check" size="sm"></icon>Saved
                  </badge>
                  <input name="value" size="sm" label="Default harness"
                    value={harnessDefault()} placeholder="e.g. claude, codex, gemini"
                    on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                    on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                </row>
              </form>
            </row>

            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>Default harness model</span>
                <span>Model passed to the launched harness when no `--model` flag is set</span>
              </column>
              <form mutation="ship.settings.update" on:success={() => flashSaved(savedHarnessModel)}>
                <input type="hidden" name="key" value="harness.default_model" />
                <row gap="2" align="center" justify="end">
                  <badge tone="success" variant="soft" size="sm" data-visible={savedHarnessModel()}>
                    <icon slot="leading" name="check" size="sm"></icon>Saved
                  </badge>
                  <input name="value" size="sm" label="Default harness model"
                    value={harnessDefaultModel()} placeholder="e.g. sonnet, opus"
                    on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                    on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                </row>
              </form>
            </row>
          </column>
        </card>

        <!-- Sessions -->
        <card padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="play" size="sm" tone="muted"></icon>
              <h2>Sessions</h2>
            </row>
          </div>
          <column gap="3">
            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span id="s-label-auto-resume">Auto-resume agent sessions</span>
                <span>When on, `ship use --launch` re-enters an existing (agent, path) session instead of spawning fresh</span>
              </column>
              <form mutation="ship.settings.update" on:success={() => flashSaved(savedAutoResume)}>
                <input type="hidden" name="key" value="agent_session.auto_resume" />
                <row gap="2" align="center" justify="end">
                  <badge tone="success" variant="soft" size="sm" data-visible={savedAutoResume()}>
                    <icon slot="leading" name="check" size="sm"></icon>Saved
                  </badge>
                  <switch name="value" size="sm" aria-labelledby="s-label-auto-resume"
                    checked={autoResume()}
                    on:sail-change={(e) => e.target.closest('form')?.requestSubmit()}></switch>
                </row>
              </form>
            </row>
          </column>
        </card>

        <!-- Advanced (collapsible) -->
        <card padding="none">
          <collapsible on:sail-toggle={() => advancedOpen.set(!advancedOpen())}>
            <div slot="trigger">
              <row gap="2" align="center">
                <icon name="settings" size="sm" tone="muted"></icon>
                <h2>Advanced</h2>
              </row>
              <icon name="chevron-down" size="sm" tone="muted" data-open={advancedOpen()}></icon>
            </div>
            <separator decorative></separator>
            <column gap="3">
              <row gap="3" justify="between" align="center">
                <span>Daemon status</span>
                <row gap="2" align="center">
                  <status-dot
                    tone={settings().kind === 'ok' ? 'success' : 'destructive'}
                    label={settings().kind === 'ok' ? 'Connected' : 'Disconnected'}></status-dot>
                  <span>{settings().kind === 'ok' ? 'Connected' : 'Disconnected'}</span>
                </row>
              </row>

              <row gap="3" justify="between" align="center">
                <span>Daemon endpoint</span>
                <code-block>{daemonEndpoint()}</code-block>
              </row>

              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Registry endpoint</span>
                  <span>Package registry URL</span>
                </column>
                <form mutation="ship.settings.update" on:success={() => flashSaved(savedRegistry)}>
                  <input type="hidden" name="key" value="registry.endpoint" />
                  <row gap="2" align="center" justify="end">
                    <badge tone="success" variant="soft" size="sm" data-visible={savedRegistry()}>
                      <icon slot="leading" name="check" size="sm"></icon>Saved
                    </badge>
                    <input name="value" size="sm" label="Registry endpoint"
                      value={registryEndpoint()} placeholder="https://registry.getship.dev"
                      on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                      on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                  </row>
                </form>
              </row>

              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Watcher debounce (ms)</span>
                  <span>Debounce window for the declarative `.ship/` file watcher</span>
                </column>
                <form mutation="ship.settings.update" on:success={() => flashSaved(savedWatcherDebounce)}>
                  <input type="hidden" name="key" value="watcher.debounce_ms" />
                  <row gap="2" align="center" justify="end">
                    <badge tone="success" variant="soft" size="sm" data-visible={savedWatcherDebounce()}>
                      <icon slot="leading" name="check" size="sm"></icon>Saved
                    </badge>
                    <input type="number" name="value" size="sm" label="Watcher debounce (ms)"
                      value={watcherDebounceMs()} placeholder="200"
                      on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                      on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                  </row>
                </form>
              </row>

              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Git log limit</span>
                  <span>Default row count for `git log` reads when callers omit a limit</span>
                </column>
                <form mutation="ship.settings.update" on:success={() => flashSaved(savedGitLogLimit)}>
                  <input type="hidden" name="key" value="git.log_limit" />
                  <row gap="2" align="center" justify="end">
                    <badge tone="success" variant="soft" size="sm" data-visible={savedGitLogLimit()}>
                      <icon slot="leading" name="check" size="sm"></icon>Saved
                    </badge>
                    <input type="number" name="value" size="sm" label="Git log limit"
                      value={gitLogLimit()} placeholder="50"
                      on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                      on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                  </row>
                </form>
              </row>

              <row gap="3" justify="between" align="center">
                <column gap="1">
                  <span>Hooks max depth</span>
                  <span>Max hook recursion depth before the engine drops further events</span>
                </column>
                <form mutation="ship.settings.update" on:success={() => flashSaved(savedHooksMaxDepth)}>
                  <input type="hidden" name="key" value="hooks.max_depth" />
                  <row gap="2" align="center" justify="end">
                    <badge tone="success" variant="soft" size="sm" data-visible={savedHooksMaxDepth()}>
                      <icon slot="leading" name="check" size="sm"></icon>Saved
                    </badge>
                    <input type="number" name="value" size="sm" label="Hooks max depth"
                      value={hooksMaxDepth()} placeholder="3"
                      on:sail-input-change={(e) => e.target.closest('form')?.requestSubmit()}
                      on:keydown={(e) => { if (e.key === 'Enter') e.target.blur() }}>
                  </row>
                </form>
              </row>
            </column>
          </collapsible>
        </card>

        <!-- About -->
        <card padding="none">
          <div slot="header">
            <row gap="2" align="center">
              <icon name="info" size="sm" tone="muted"></icon>
              <h2>About</h2>
            </row>
          </div>
          <column gap="3">
            <row gap="3" justify="between" align="center">
              <span>Documentation</span>
              <a href="https://getship.dev/docs" external>getship.dev/docs</a>
            </row>
            <row gap="3" justify="between" align="center">
              <span>Source</span>
              <a href="https://github.com/madvib/Ship" external>GitHub</a>
            </row>
            <row gap="3" justify="between" align="center">
              <column gap="1">
                <span>First-run setup</span>
                <span>Run the welcome wizard again</span>
              </column>
              <button variant="outline" size="sm" data-test="settings-rerun-setup"
                on:click={rerunSetup}>
                Run setup again
              </button>
            </row>
          </column>
        </card>

      </column>

    </if>
  </main>
</div>
