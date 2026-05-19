import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-command-palette/sui-command-palette.compiled.js'
import '../src/sui-icon/sui-icon.compiled.js'
import '../src/sui-spinner/sui-spinner.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'
import type { CommandGroup } from '../src/sui-command-palette/sui-command-palette.controller'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const dialogOf = (el: HTMLElement) => shadow(el).querySelector('dialog') as HTMLDialogElement
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input[role="combobox"]') as HTMLInputElement
const rows = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('button.row')) as HTMLButtonElement[]
const headers = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('.group-h')).map((h) => h.textContent?.trim())

const GROUPS: CommandGroup[] = [
  {
    label: 'Open',
    items: [
      { id: 'studio', label: 'Open Studio', icon: 'grid', hotkey: '⌘.', value: '/studio' },
      { id: 'sessions', label: 'Open Sessions', icon: 'activity', value: '/sessions' },
      { id: 'events', label: 'Open Events', icon: 'radio', value: '/events' },
    ],
  },
  {
    label: 'Run',
    items: [
      { id: 'a:rust', label: 'Run agent rust-compiler', icon: 'bot', value: 'run:agent:rust' },
      { id: 's:tdd', label: 'Invoke skill tdd', icon: 'zap', value: 'run:skill:tdd' },
    ],
  },
  {
    label: 'Settings',
    items: [
      { id: 'theme', label: 'Switch theme…', icon: 'palette', value: 'action:theme' },
    ],
  },
]

function setup(groups: CommandGroup[] = GROUPS): HTMLElement & { commands: CommandGroup[]; show: () => void } {
  const { el } = mount(reg, 'sui-command-palette')
  const palette = el as HTMLElement & { commands: CommandGroup[]; show: () => void }
  palette.commands = groups
  return palette
}

describe('<sui-command-palette>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-command-palette')).toBeDefined()
  })

  it('renders a dialog with a combobox input', async () => {
    const el = setup()
    await flush()
    expect(dialogOf(el)).not.toBeNull()
    const inp = inputOf(el)
    expect(inp).not.toBeNull()
    expect(inp.getAttribute('aria-controls')).toBeTruthy()
  })

  it('renders the three groups with their items', async () => {
    const el = setup()
    await flush()
    expect(headers(el)).toEqual(['Open', 'Run', 'Settings'])
    expect(rows(el).length).toBe(6)
  })

  it('open attribute reflects + showModal is invoked', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('show() method opens the dialog', async () => {
    const el = setup()
    el.show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('filters rows by case-insensitive substring across label + meta', async () => {
    const el = setup()
    await flush()
    const inp = inputOf(el)
    inp.value = 'RUN'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    // Only the "Run agent rust-compiler" row contains "run".
    const labels = rows(el).map((r) => r.querySelector('.row-label')?.textContent?.trim())
    expect(labels).toEqual(['Run agent rust-compiler'])
    expect(headers(el)).toEqual(['Run'])
  })

  it('shows an empty state when nothing matches', async () => {
    const el = setup()
    await flush()
    const inp = inputOf(el)
    inp.value = 'zzzznomatch'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(rows(el).length).toBe(0)
    expect(shadow(el).querySelector('.empty')?.textContent).toContain('No matching')
  })

  it('first row is highlighted by default; ArrowDown moves selection', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    expect(rows(el)[0]!.classList.contains('hl')).toBe(true)
    const dialog = dialogOf(el)
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await flush()
    expect(rows(el)[0]!.classList.contains('hl')).toBe(false)
    expect(rows(el)[1]!.classList.contains('hl')).toBe(true)
  })

  it('ArrowUp from the first row wraps to the last', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    const dialog = dialogOf(el)
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    await flush()
    const last = rows(el).length - 1
    expect(rows(el)[last]!.classList.contains('hl')).toBe(true)
  })

  it('Enter activates the highlighted row → sui-command-select', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    const fired: CustomEvent[] = []
    el.addEventListener('sui-command-select', (e) => fired.push(e as CustomEvent))
    dialogOf(el).dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await flush()
    dialogOf(el).dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(fired.length).toBe(1)
    expect(fired[0]!.detail).toEqual({ id: 'sessions', value: '/sessions' })
    // Activating closes the palette.
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('clicking a row fires sui-command-select with id + value', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    const fired: CustomEvent[] = []
    el.addEventListener('sui-command-select', (e) => fired.push(e as CustomEvent))
    rows(el)[0]!.click()
    await flush()
    expect(fired.length).toBe(1)
    expect(fired[0]!.detail).toEqual({ id: 'studio', value: '/studio' })
  })

  it('Escape emits sui-command-close and closes', async () => {
    const el = setup()
    el.setAttribute('open', '')
    await flush()
    const fired: Event[] = []
    el.addEventListener('sui-command-close', (e) => fired.push(e))
    dialogOf(el).dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(fired.length).toBe(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('reading el.commands round-trips the assigned groups', async () => {
    const el = setup() as HTMLElement & { commands: CommandGroup[] }
    await flush()
    expect(el.commands.map((g) => g.label)).toEqual(['Open', 'Run', 'Settings'])
  })

  it('shows a spinner when loading and no commands set yet', async () => {
    const { el } = mount(reg, 'sui-command-palette', { loading: true })
    await flush()
    expect(shadow(el).querySelector('.spinner-wrap')).not.toBeNull()
  })
})
