import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

export default controller<Scope>({
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
