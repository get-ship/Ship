<controller>
import { controller, signal, derived } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

type WizardStep = 'welcome' | 'step1' | 'step2' | 'step3'

interface GlyphOption { id: string; label: string }
interface AccentOption { id: string; label: string }
interface ThemeOption { id: string; label: string }
interface NavOption { id: string; label: string }
interface DockOption { id: string; label: string }

const GLYPHS: GlyphOption[] = [
  { id: 'S', label: 'S' }, { id: 'P', label: 'P' },
  { id: 'M', label: 'M' }, { id: 'L', label: 'L' },
  { id: 'V', label: 'V' }, { id: 'bolt', label: '⚡' },
  { id: 'diamond', label: '♦' }, { id: 'triangle', label: '▲' },
  { id: 'spade', label: '♠' }, { id: 'half', label: '◑' },
]

const ACCENTS: AccentOption[] = [
  { id: 'red', label: 'Red' }, { id: 'crimson', label: 'Crimson' },
  { id: 'green', label: 'Green' }, { id: 'blue', label: 'Blue' },
  { id: 'purple', label: 'Purple' }, { id: 'indigo', label: 'Indigo' },
  { id: 'slate', label: 'Slate' }, { id: 'orange', label: 'Orange' },
]

const THEMES: ThemeOption[] = [
  { id: 'light', label: 'Light' }, { id: 'dark', label: 'Dark' },
  { id: 'paper', label: 'Paper' }, { id: 'halo', label: 'Halo' },
  { id: 'console', label: 'Console' }, { id: 'contrast', label: 'Contrast' },
]

const NAVS: NavOption[] = [
  { id: 'rail', label: 'rail' }, { id: 'top', label: 'top' }, { id: 'none', label: 'none' },
]

const DOCKS: DockOption[] = [
  { id: 'left', label: 'left' }, { id: 'centered', label: 'centered' },
  { id: 'right', label: 'right' }, { id: 'off', label: 'off' },
]

const PREV_STEP: Record<WizardStep, WizardStep> = {
  welcome: 'welcome', step1: 'welcome', step2: 'step1', step3: 'step2',
}

export default controller({
  scope: () => {
    const step = signal<WizardStep>('welcome')
    const name = signal('Brand Voice')
    const glyph = signal('diamond')
    const accent = signal('green')
    const theme = signal('halo')
    const navigation = signal('rail')
    const dock = signal('centered')

    const glyphs = derived(() => GLYPHS)
    const accents = derived(() => ACCENTS)
    const themes = derived(() => THEMES)
    const navs = derived(() => NAVS)
    const docks = derived(() => DOCKS)

    const slug = derived(() =>
      name().toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, ''),
    )
    const currentGlyphLabel = derived(
      () => GLYPHS.find((g) => g.id === glyph())?.label ?? '♦',
    )

    // Handlers as typed closures — avoids the writable-proxy `as never` casts
    // that arise when mutating scope signals through the on: block's untyped proxy.
    const goToStep1 = () => step.set('step1')
    const goToStep2 = () => step.set('step2')
    const goToStep3 = () => step.set('step3')
    const goBack = () => step.set(PREV_STEP[step()] ?? 'welcome')
    const setGlyph = (id: string) => glyph.set(id)
    const setAccent = (id: string) => accent.set(id)
    const setTheme = (id: string) => theme.set(id)
    const setNavigation = (id: string) => navigation.set(id)
    const setDock = (id: string) => dock.set(id)
    const skip = () => {}
    const done = () => {}
    const launch = () => {}

    return {
      step, name, glyph, accent, theme, navigation, dock,
      glyphs, accents, themes, navs, docks,
      slug, currentGlyphLabel,
      goToStep1, goToStep2, goToStep3, goBack,
      setGlyph, setAccent, setTheme, setNavigation, setDock,
      skip, done, launch,
    }
  },
})
</controller>

<div>

  <match on={step()}>

    <case of={'welcome'}>
    <div>
      <header>
        <div aria-hidden="true">S</div>
        <div>
          <h1>Welcome to Ship.</h1>
          <p>v1.4.2 &middot; macOS &middot; hub installed</p>
        </div>
      </header>
      <p>
        Ship lives in your applications dock. Inside Ship, every <strong>space</strong> you
        create becomes its own app &mdash; its own window, its own theme, its own icon in your OS launcher.
      </p>
      <dl>
        <div>
          <dt>Ship hub</dt>
          <dd>this app &mdash; manage spaces, account, marketplace, system settings, activity.</dd>
        </div>
        <div>
          <dt>Spaces</dt>
          <dd>each one runs in its own window. Switch between them like any other app.</dd>
        </div>
        <div>
          <dt>Studio</dt>
          <dd>inside the hub &mdash; customize a space's theme, dock, chrome.</dd>
        </div>
      </dl>
      <div role="note">
        <span aria-hidden="true"></span>
        <p>
          Heads-up: Ship will register each space as an application on your system
          (Applications &middot; Start Menu &middot; Activities). You can uninstall them from the hub.
        </p>
      </div>
      <footer>
        <button type="button" on:click={skip}>Skip &mdash; explore the hub</button>
        <button type="button" on:click={goToStep1}>Create your first space</button>
      </footer>
    </div>

    </case><case of={'step1'}>
    <div>
      <div>
        <span>STEP 1</span>
        <div role="progressbar" aria-valuenow="33" aria-valuemin="0" aria-valuemax="100">
          <div></div>
        </div>
      </div>
      <h1>Name your space.</h1>
      <div aria-label="Preview">
        <div data-accent={accent()}>
          <span aria-hidden="true">{currentGlyphLabel()}</span>
        </div>
        <div>
          <div>{name() || 'Brand Voice'}</div>
          <div>~/Spaces/{slug() || 'brand-voice'}</div>
        </div>
        <span>PREVIEW</span>
      </div>
      <label for="cs-name-input">NAME</label>
      <input id="cs-name-input" type="text" placeholder="Brand Voice" bind:value={name} />
      <div>GLYPH &amp; ACCENT</div>
      <div role="group" aria-label="Choose glyph">
        <for as="g" of={glyphs()} key={g.id}>
          <button
            data-cs-glyph-tile--selected={glyph() === g.id}
            type="button"
            aria-pressed={glyph() === g.id ? 'true' : 'false'}
            data-accent={glyph() === g.id ? accent() : null}
            on:click={() => setGlyph(g.id)}>
            {g.label}
          </button>
        </for>
      </div>
      <div role="group" aria-label="Choose accent color">
        <for as="a" of={accents()} key={a.id}>
          <button
            data-cs-accent-swatch--selected={accent() === a.id}
            type="button"
            aria-pressed={accent() === a.id ? 'true' : 'false'}
            data-accent={a.id}
            aria-label={a.label}
            on:click={() => setAccent(a.id)}>
          </button>
        </for>
      </div>
      <footer>
        <button type="button" on:click={goBack}>&larr; Back</button>
        <button type="button" on:click={goToStep2}>Continue</button>
      </footer>
    </div>

    </case><case of={'step2'}>
    <div>
      <div>
        <span>STEP 2</span>
        <div role="progressbar" aria-valuenow="67" aria-valuemin="0" aria-valuemax="100">
          <div></div>
        </div>
      </div>
      <h1>Pick its look.</h1>
      <div>
        <div>
          <span>THEME</span>
          <span>7 built-in &middot; marketplace has more</span>
        </div>
        <div role="group" aria-label="Choose theme">
          <for as="t" of={themes()} key={t.id}>
            <button
              data-cs-theme-card--selected={theme() === t.id}
              type="button"
              aria-pressed={theme() === t.id ? 'true' : 'false'}
              data-theme={t.id}
              on:click={() => setTheme(t.id)}>
              <div data-theme={t.id}>
                <div></div>
                <div>
                  <div></div>
                  <div></div>
                </div>
              </div>
              <span>{t.label}</span>
            </button>
          </for>
        </div>
      </div>
      <div>
        <div>
          <span>NAVIGATION</span>
          <div role="group" aria-label="Choose navigation style">
            <for as="n" of={navs()} key={n.id}>
              <button
                data-cs-option-card--selected={navigation() === n.id}
                type="button"
                aria-pressed={navigation() === n.id ? 'true' : 'false'}
                on:click={() => setNavigation(n.id)}>
                <div data-nav={n.id}></div>
                <span>{n.label}</span>
              </button>
            </for>
          </div>
        </div>
        <div>
          <span>DOCK</span>
          <div role="group" aria-label="Choose dock position">
            <for as="d" of={docks()} key={d.id}>
              <button
                data-cs-option-card--selected={dock() === d.id}
                type="button"
                aria-pressed={dock() === d.id ? 'true' : 'false'}
                on:click={() => setDock(d.id)}>
                <div data-dock={d.id}></div>
                <span>{d.label}</span>
              </button>
            </for>
          </div>
        </div>
      </div>
      <p><span>tip</span> You can change all of this later, in Studio.</p>
      <footer>
        <button type="button" on:click={goBack}>&larr; Back</button>
        <button type="button" on:click={goToStep3}>Continue</button>
      </footer>
    </div>

    </case><case of={'step3'}>
    <div>
      <div>
        <span>STEP 3</span>
        <div role="progressbar" aria-valuenow="100" aria-valuemin="0" aria-valuemax="100">
          <div></div>
        </div>
      </div>
      <h1>{name() || 'Your space'} is installed.</h1>
      <div>
        <div data-accent={accent()}>
          <span aria-hidden="true">{currentGlyphLabel()}</span>
        </div>
        <div>
          <div>{name() || 'Brand Voice'}</div>
          <div>{theme()} &middot; {navigation()} &middot; ~/Spaces/{slug()}</div>
        </div>
        <span aria-hidden="true"></span>
      </div>
      <div>
        <div>WHERE YOU'LL FIND IT</div>
        <div role="list">
          <div role="listitem">
            <span aria-hidden="true">&#10003;</span>
            <strong>Applications</strong>
            <span>macOS dock &mdash; drag to keep it there</span>
          </div>
          <div role="listitem">
            <span aria-hidden="true">&#10003;</span>
            <strong>Spotlight</strong>
            <span>&#8984; Space &rarr; &ldquo;{name() || 'Brand Voice'}&rdquo;</span>
          </div>
          <div role="listitem">
            <span aria-hidden="true">&#10003;</span>
            <strong>Ship hub</strong>
            <span>Spaces &middot; marked as installed</span>
          </div>
          <div role="listitem">
            <span aria-hidden="true">&#10003;</span>
            <strong>Shell PATH</strong>
            <span>ship open {slug() || 'brand-voice'}</span>
          </div>
        </div>
      </div>
      <div role="note">
        <span>note</span>
        Uninstall any space from Hub &rarr; Spaces &rarr; &hellip;&ensp;Ship removes the host-OS entries it created.
      </div>
      <footer>
        <button type="button" on:click={done}>Done</button>
        <button type="button" on:click={launch}>Launch {name() || 'Space'}</button>
      </footer>
    </div>

    </case><default>
    <div></div>

  </default></match>

</div>
