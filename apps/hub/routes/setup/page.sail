<controller>
// Hub `/setup` route controller — first-run onboarding wizard.
//
// Ported from the shell's `/setup` chrome. State lives here; `page.sui` only
// declares the six steps it renders. Rebuilt on the hub's ambient
// `ctx.ops` unary client (the host injects it via `entry.gen.ts`; the
// account route uses the same seam).
//
// Contract: `ops.space.add({ path, name })` — `SpaceService.Add`
// (`AddSpaceReq.path/name`), already on the Connect surface. The provider step
// records a choice ONLY; Ship validates provider CLIs in Settings (the shell's
// `/api/v1/harnesses/installed` PATH-detection probe is off-contract — logged
// as a REQ in docs/framework-issues.md, so the inert Validating/Ready/error
// scaffolding it drove is gone, not parked behind a dead branch).
import { controller, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const STORAGE_KEY = 'ship-setup-state'
const TOTAL_STEPS = 6
const PROVIDERS = [
  { id: 'claude',   name: 'Claude',   sub: 'Anthropic · claude' },
  { id: 'codex',    name: 'Codex',    sub: 'OpenAI · codex' },
  { id: 'gemini',   name: 'Gemini',   sub: 'Google · gemini' },
  { id: 'opencode', name: 'OpenCode', sub: 'Open source · opencode' },
  { id: 'cursor',   name: 'Cursor',   sub: 'Cursor · cursor-agent' },
] as const

// Structural type for the one op the wizard calls — the full Connect client
// (`s.clients.ops`) is assignable to it.
type SpaceClient = { space: { add(req: { path: string; name: string }): Promise<unknown> } }

interface PersistedState {
  step: number
  identityName: string
  provider: string
  projectName: string
  projectPath: string
  sampleApp: boolean
  completed: boolean
}

function defaultProjectPath(name: string): string {
  const trimmed = name.trim() || '<name>'
  return `~/Documents/ShipProjects/${trimmed}`
}

function loadPersisted(): Partial<PersistedState> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? parsed : {}
  } catch {
    return {}
  }
}

function savePersisted(state: PersistedState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch { /* best-effort */ }
}

function readEntry(s: Record<string, unknown>): PersistedState {
  // `s` is the MutableState proxy: indexing a state key returns the UNWRAPPED
  // value (the proxy already calls the signal). Read it directly — do NOT call
  // it (the old `(s[k])()` invoked a number/string and crashed at runtime).
  const sig = <T>(k: string): T => s[k] as T
  return {
    step: sig<number>('step'),
    identityName: sig<string>('identityName'),
    provider: sig<string>('provider'),
    projectName: sig<string>('projectName'),
    projectPath: sig<string>('projectPath'),
    sampleApp: sig<boolean>('sampleApp'),
    completed: false,
  }
}

async function postProject(ops: SpaceClient, name: string, path: string): Promise<{ ok: boolean; error?: string; warning?: string }> {
  try {
    await ops.space.add({ path, name })
    return { ok: true }
  } catch (e) {
    const msg = (e as Error).message ?? 'add failed'
    if (/not a directory|exists/i.test(msg)) {
      return { ok: false, warning: msg }
    }
    return { ok: false, error: msg }
  }
}

function persistFromState(s: Record<string, unknown>, completed = false): void {
  const entry = readEntry(s)
  entry.completed = completed
  savePersisted(entry)
}

// Step-transition behaviors live at module scope so both the `on:` handlers
// (template-wired) and the keyboard shortcuts (mount / onFieldKeydown) invoke
// the SAME function. `on:` handlers are not members of the state proxy, so a
// handler cannot reach a sibling handler through `s.*`; these shared functions
// take the same `Record<string, unknown>` view `persistFromState`/`readEntry`
// already accept (the proxy IS that shape).
function advanceStep(s: Record<string, unknown>): void {
  const next = Math.min(TOTAL_STEPS, (s.step as number) + 1)
  s.step = next
  persistFromState(s)
}

function submitIdentityStep(s: Record<string, unknown>): void {
  const name = String(s.identityName ?? '').trim()
  if (!name) {
    s.identityError = 'Please enter a display name.'
    return
  }
  s.identityError = ''
  s.step = 3
  persistFromState(s)
}

function submitProjectStep(s: Record<string, unknown>, ops: SpaceClient): void {
  const name = String(s.projectName ?? '').trim()
  if (!name) return
  const path = String(s.projectPath ?? '').trim() || defaultProjectPath(name)
  s.projectPath = path
  s.projectError = ''
  s.projectWarning = ''
  s.projectSubmitting = true
  void (async () => {
    const result = await postProject(ops, name, path)
    s.projectSubmitting = false
    if (result.ok) {
      s.step = 5
      persistFromState(s)
      return
    }
    if (result.warning) {
      s.projectWarning = result.warning
    } else if (result.error) {
      s.projectError = result.error
    }
    // Advance anyway — capture-now, fix-in-settings.
    s.step = 5
    persistFromState(s)
  })()
}

function submitSampleStep(s: Record<string, unknown>): void {
  s.step = 6
  persistFromState(s)
}

function finishSetup(s: Record<string, unknown>): void {
  persistFromState(s, true)
  try { localStorage.setItem('ship-setup-completed', '1') } catch { /* best-effort */ }
  window.location.href = '/'
}

// Listener teardown ref (set in mount, called in unmount). Module-local
// because the chrome scope proxy rejects extra property writes.
let setupKeyCleanup: (() => void) | null = null

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    void ctx
    const persisted = loadPersisted()
    return {
      step: signal<number>(persisted.completed ? 1 : (persisted.step ?? 1)),
      open: signal<boolean>(true),
      identityName: signal<string>(persisted.identityName ?? ''),
      identityError: signal<string>(''),
      provider: signal<string>(persisted.provider ?? ''),
      projectName: signal<string>(persisted.projectName ?? ''),
      projectPath: signal<string>(persisted.projectPath ?? ''),
      projectError: signal<string>(''),
      projectWarning: signal<string>(''),
      projectSubmitting: signal<boolean>(false),
      sampleApp: signal<boolean>(persisted.sampleApp ?? true),
      confirmingExit: signal<boolean>(false),
      providerList: PROVIDERS,
    }
  },
  on: {
    goNext: (s) => advanceStep(s),
    goBack: (s) => {
      const prev = Math.max(1, s.step - 1)
      s.step = prev
      persistFromState(s)
    },
    submitIdentity: (s) => submitIdentityStep(s),
    selectProvider: (s, id: string) => {
      s.provider = id
      persistFromState(s)
    },
    submitProvider: (s) => {
      // Provider choice is captured; credentials/CLI detection happen in
      // Settings (no PATH-detection op on the current contract).
      s.step = 4
      persistFromState(s)
    },
    submitProject: (s) => submitProjectStep(s, s.clients.ops),
    submitSample: (s) => submitSampleStep(s),
    finish: (s) => finishSetup(s),
    requestExit: (s) => { s.confirmingExit = true },
    cancelExit: (s) => { s.confirmingExit = false },
    confirmExit: (s) => {
      s.confirmingExit = false
      persistFromState(s, true)
      try { localStorage.setItem('ship-setup-completed', '1') } catch { /* best-effort */ }
      window.location.href = '/'
    },
    onFieldKeydown: (s, e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        const target = e.target as (HTMLInputElement & { blur(): void }) | null
        target?.blur()
        e.stopPropagation()
        return
      }
      if (e.key === 'Enter') {
        e.preventDefault()
        const step = s.step
        if (step === 2) submitIdentityStep(s)
        else if (step === 4) submitProjectStep(s, s.clients.ops)
      }
    },
  },
  mount: (state, _el) => {
    // Cmd+. (or Ctrl+.) dismisses the wizard with confirmation.
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === '.') {
        e.preventDefault()
        state.confirmingExit = true
        return
      }
      // Top-level Enter advances the welcome / sample / done steps.
      if (e.key === 'Enter' && !(e.target instanceof HTMLInputElement)) {
        const step = state.step
        if (step === 1) advanceStep(state)
        else if (step === 5) submitSampleStep(state)
        else if (step === 6) finishSetup(state)
      }
    }
    window.addEventListener('keydown', onKey, true)
    // Chrome scope is a guarded proxy — can't stash _cleanup on it the way the
    // daemon view host allowed. Keep the listener teardown module-local.
    setupKeyCleanup = () => window.removeEventListener('keydown', onKey, true)
  },
  unmount: () => {
    setupKeyCleanup?.()
    setupKeyCleanup = null
  },
})
</controller>

<!--
  Hub `/setup` — first-run onboarding wizard.

  Ported from the shell's `/setup` chrome. State lives in `controller.ts`;
  this template only declares the six steps it renders. Page styling lives in
  the co-located `style.css` (deck-route-css convention) — never an inline
  <style> (FW-4: the bundler silently drops a route-level <style> block). Tokens
  only — no hardcoded colors / spacing / radii.

  Page chrome: the top nav is owned by the root layout (`<ship-app-chrome>`);
  the wizard renders inside a backdrop-blur `<dialog>` that overlays it.
-->
<div data-test="setup-view">
  <dialog
    open={open()}
    backdrop="blur"
    size="lg"
    no-close-button
    no-header
    no-footer
    dismissible="false">

    <div data-test="setup-wizard" data-step={step()}>

      <header>
        <ol role="progressbar" aria-label="Setup progress" aria-valuenow={step()} aria-valuemin="1" aria-valuemax="6">
          <for as="stepNum" of={[1, 2, 3, 4, 5, 6]} key={stepNum}>
            <li data-current={step() === stepNum} data-done={step() > stepNum}></li>
          </for>
        </ol>
        <button variant="ghost" size="sm" data-test="setup-skip"
          on:click={requestExit}>
          Skip setup
        </button>
      </header>

      <!-- Step 1: Welcome -->
      <if cond={step() === 1}>
        <section data-test="step-welcome">
          <p>Step 1 of 6</p>
          <h1>Welcome to Ship</h1>
          <p>
            Ship is the workbench for shipping software with agents. It runs
            locally — you own the data, the agents, and the pipelines.
          </p>
          <p>
            The next five steps set up an identity, connect a provider, and
            create your first project. You can change any of it later.
          </p>
          <div>
            <button tone="primary" variant="solid" size="md"
              data-test="step-welcome-next"
              on:click={goNext}>
              Let's set it up
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

      <!-- Step 2: Identity -->
      <if cond={step() === 2}>
        <section data-test="step-identity">
          <p>Step 2 of 6</p>
          <h2>What should we call you?</h2>
          <p>
            Your display name shows up next to agent activity, sessions, and
            git commits Ship makes on your behalf.
          </p>
          <field>
            <label for="setup-identity-name">Display name</label>
            <input id="setup-identity-name" size="md"
              placeholder="e.g. Ada Lovelace"
              data-test="step-identity-name"
              bind:value={identityName}
              on:keydown={onFieldKeydown}>
          </field>
          <if cond={identityError()}>
            <alert tone="destructive" data-test="step-identity-error">
              {identityError()}
            </alert>
          </if>
          <div>
            <button variant="ghost" size="md" on:click={goBack}>
              <icon slot="leading" name="arrow-left"></icon>Back
            </button>
            <button tone="primary" variant="solid" size="md"
              data-test="step-identity-next"
              on:click={submitIdentity}>
              Continue
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

      <!-- Step 3: Provider -->
      <if cond={step() === 3}>
        <section data-test="step-provider">
          <p>Step 3 of 6</p>
          <h2>Connect a provider</h2>
          <p>
            Ship drives whichever provider CLIs you have installed. Pick one
            to start with — add more later from Settings.
          </p>
          <div role="radiogroup" aria-label="AI provider">
            <for as="p" of={providerList} key={p.id}>
              <card interactive padding="md" role="radio"
                data-test={'provider-' + p.id}
                aria-checked={provider() === p.id ? 'true' : 'false'}
                tone={provider() === p.id ? 'primary' : ''}
                fill={provider() === p.id}
                on:click={() => selectProvider(p.id)}>
                <div>
                  <div>
                    <div>{p.name}</div>
                    <div>{p.sub}</div>
                  </div>
                </div>
              </card>
            </for>
          </div>
          <if cond={provider()}>
            <p data-test="provider-note">
              <icon name="check" size="sm"></icon>
              Choice saved. Ship validates provider CLIs in Settings.
            </p>
          </if>
          <div>
            <button variant="ghost" size="md" on:click={goBack}>
              <icon slot="leading" name="arrow-left"></icon>Back
            </button>
            <button tone="primary" variant="solid" size="md"
              data-test="step-provider-next"
              disabled={!provider()}
              on:click={submitProvider}>
              Continue
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

      <!-- Step 4: Project -->
      <if cond={step() === 4}>
        <section data-test="step-project">
          <p>Step 4 of 6</p>
          <h2>Create your first project</h2>
          <p>
            A project is a folder Ship watches. Agents and pipelines work
            inside it; you can have many.
          </p>
          <field>
            <label for="setup-project-name">Project name</label>
            <input id="setup-project-name" size="md"
              placeholder="my-first-project"
              data-test="step-project-name"
              bind:value={projectName}
              on:keydown={onFieldKeydown}>
          </field>
          <field>
            <label for="setup-project-path">Location</label>
            <input id="setup-project-path" size="md"
              placeholder="~/Documents/ShipProjects/<name>"
              data-test="step-project-path"
              bind:value={projectPath}
              on:keydown={onFieldKeydown}>
            <span slot="hint">
              Defaults to <code-block>~/Documents/ShipProjects/&lt;name&gt;</code-block>.
              The folder is created if it doesn't exist.
            </span>
          </field>
          <if cond={projectWarning()}>
            <alert tone="warning" data-test="project-warning">
              {projectWarning()}
            </alert>
          </if>
          <if cond={projectError()}>
            <alert tone="destructive" data-test="project-error">
              {projectError()}
            </alert>
          </if>
          <div>
            <button variant="ghost" size="md" on:click={goBack}>
              <icon slot="leading" name="arrow-left"></icon>Back
            </button>
            <button tone="primary" variant="solid" size="md"
              data-test="step-project-next"
              loading={projectSubmitting()}
              disabled={!projectName().length}
              on:click={submitProject}>
              Create project
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

      <!-- Step 5: Sample app -->
      <if cond={step() === 5}>
        <section data-test="step-sample">
          <p>Step 5 of 6</p>
          <h2>Want a sample app to start with?</h2>
          <p>
            Scaffolds a "Hello, Ship" view in your project so the first time
            you open it isn't empty. Easy to delete later.
          </p>
          <field>
            <input type="checkbox"
              data-test="step-sample-toggle"
              bind:checked={sampleApp}>
              Add the Hello, Ship sample app
            
          </field>
          <div>
            <button variant="ghost" size="md" on:click={goBack}>
              <icon slot="leading" name="arrow-left"></icon>Back
            </button>
            <button tone="primary" variant="solid" size="md"
              data-test="step-sample-next"
              on:click={submitSample}>
              Continue
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

      <!-- Step 6: Done -->
      <if cond={step() === 6}>
        <section data-test="step-done">
          <p>Step 6 of 6</p>
          <h2>Ship is ready</h2>
          <p>
            Hi {identityName() || 'there'}, your project
            <strong>{projectName()}</strong> is set up
            <if cond={sampleApp()}>with the Hello, Ship sample app</if>.
          </p>
          <p>
            Jump in — the rail on the left navigates to Agents, Pipelines,
            and the rest of Ship.
          </p>
          <div>
            <button tone="primary" variant="solid" size="lg"
              data-test="step-done-finish"
              on:click={finish}>
              Open Ship
              <icon slot="trailing" name="arrow-right"></icon>
            </button>
          </div>
        </section>
      </if>

    </div>
  </dialog>

  <alert-dialog
    open={confirmingExit()}
    tone="default"
    confirm-label="Skip setup"
    cancel-label="Keep going"
    data-test="setup-confirm-exit"
    on:sail-confirm={confirmExit}
    on:sail-cancel={cancelExit}>
    <span slot="title">Skip setup?</span>
    <p>
      We'll apply sensible defaults for the steps you haven't filled in.
      You can run setup again from Settings → About.
    </p>
  </alert-dialog>
</div>
