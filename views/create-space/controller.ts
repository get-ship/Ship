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
  { id: 'red', label: 'Red' },
  { id: 'crimson', label: 'Crimson' },
  { id: 'green', label: 'Green' },
  { id: 'blue', label: 'Blue' },
  { id: 'purple', label: 'Purple' },
  { id: 'indigo', label: 'Indigo' },
  { id: 'slate', label: 'Slate' },
  { id: 'orange', label: 'Orange' },
]

const THEMES: ThemeOption[] = [
  { id: 'light', label: 'Light' },
  { id: 'dark', label: 'Dark' },
  { id: 'paper', label: 'Paper' },
  { id: 'halo', label: 'Halo' },
  { id: 'console', label: 'Console' },
  { id: 'contrast', label: 'Contrast' },
]

const NAVS: NavOption[] = [
  { id: 'rail', label: 'rail' },
  { id: 'top', label: 'top' },
  { id: 'none', label: 'none' },
]

const DOCKS: DockOption[] = [
  { id: 'left', label: 'left' },
  { id: 'centered', label: 'centered' },
  { id: 'right', label: 'right' },
  { id: 'off', label: 'off' },
]

const PREV_STEP: Record<WizardStep, WizardStep> = {
  welcome: 'welcome',
  step1: 'welcome',
  step2: 'step1',
  step3: 'step2',
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

    return {
      step, name, glyph, accent, theme, navigation, dock,
      glyphs, accents, themes, navs, docks,
      slug, currentGlyphLabel,
    }
  },

  on: {
    goToStep1: (s) => { s.step = 'step1' as never },
    goToStep2: (s) => { s.step = 'step2' as never },
    goToStep3: (s) => { s.step = 'step3' as never },
    goBack: (s) => {
      const cur = s.step as unknown as WizardStep
      s.step = (PREV_STEP[cur] ?? 'welcome') as never
    },
    setGlyph: (s, id: string) => { s.glyph = id as never },
    setAccent: (s, id: string) => { s.accent = id as never },
    setTheme: (s, id: string) => { s.theme = id as never },
    setNavigation: (s, id: string) => { s.navigation = id as never },
    setDock: (s, id: string) => { s.dock = id as never },
    skip: () => {},
    done: () => {},
    launch: () => {},
  },
})
