import { afterEach, describe, expect, it } from 'vitest'
import '../../src/components/sui-view-chrome/sui-view-chrome.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()

const SHELL_IDS = [
  'view-topbar-actions',
  'view-status-start',
  'view-status-end',
  'breadcrumbs-title',
  'status-branch',
  'status-ab',
  'status-dot',
] as const

type ShellId = (typeof SHELL_IDS)[number]
type Shell = Record<ShellId, HTMLElement>

const shellEls: HTMLElement[] = []

function setupShell(): Shell {
  const map = {} as Shell
  for (const id of SHELL_IDS) {
    const el = document.createElement('div')
    el.id = id
    document.body.appendChild(el)
    shellEls.push(el)
    map[id] = el
  }
  return map
}

afterEach(() => {
  reg.cleanup()
  for (const el of shellEls.splice(0)) el.remove()
})

describe('<sui-view-chrome>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-view-chrome')).toBeDefined()
  })

  it('attaches a shadow root (component uses shadow DOM)', () => {
    const { el } = mount(reg, 'sui-view-chrome')
    expect(el.shadowRoot).not.toBeNull()
  })

  it('no-op with no shell present — slot children stay as host descendants', async () => {
    const { el } = mount(reg, 'sui-view-chrome', {}, '<button slot="actions">Go</button>')
    await flush()
    expect(el.querySelector('[slot="actions"]')).not.toBeNull()
  })

  it('moves actions slot children into #view-topbar-actions', async () => {
    const shell = setupShell()
    const { el } = mount(reg, 'sui-view-chrome', {}, '<button slot="actions">Refresh</button>')
    await flush()
    expect(shell['view-topbar-actions'].querySelector('button')).not.toBeNull()
    expect(shell['view-topbar-actions'].querySelector('button')!.textContent).toBe('Refresh')
    expect(el.querySelector('[slot="actions"]')).toBeNull()
  })

  it('moves status-start slot children into #view-status-start', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', {}, '<span slot="status-start">●</span>')
    await flush()
    expect(shell['view-status-start'].querySelector('span')).not.toBeNull()
  })

  it('moves status-end slot children into #view-status-end', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', {}, '<span slot="status-end">22 changed</span>')
    await flush()
    expect(shell['view-status-end'].querySelector('span')).not.toBeNull()
  })

  it('writes title prop to #breadcrumbs-title', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { title: 'Git' })
    await flush()
    expect(shell['breadcrumbs-title'].textContent).toBe('Git')
  })

  it('writes branch prop to #status-branch', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { branch: 'main' })
    await flush()
    expect(shell['status-branch'].textContent).toBe('main')
  })

  it('writes ahead to #status-ab as ↑N', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { ahead: '3' })
    await flush()
    expect(shell['status-ab'].textContent).toBe('↑3')
  })

  it('writes behind to #status-ab as ↓N', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { behind: '2' })
    await flush()
    expect(shell['status-ab'].textContent).toBe('↓2')
  })

  it('writes both ahead and behind to #status-ab', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { ahead: '1', behind: '4' })
    await flush()
    expect(shell['status-ab'].textContent).toBe('↑1 ↓4')
  })

  it('sets data-tone on #status-dot for non-default tone', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { tone: 'success' })
    await flush()
    expect(shell['status-dot'].dataset.tone).toBe('success')
  })

  it('sets data-busy on #status-dot when busy', async () => {
    const shell = setupShell()
    mount(reg, 'sui-view-chrome', { busy: true })
    await flush()
    expect(shell['status-dot'].hasAttribute('data-busy')).toBe(true)
  })

  it('removes moved nodes from shell on disconnect', async () => {
    const shell = setupShell()
    const { el } = mount(reg, 'sui-view-chrome', {}, '<button slot="actions">Go</button>')
    await flush()
    expect(shell['view-topbar-actions'].querySelector('button')).not.toBeNull()
    el.remove()
    expect(shell['view-topbar-actions'].querySelector('button')).toBeNull()
  })

  it('clears #breadcrumbs-title on disconnect', async () => {
    const shell = setupShell()
    const { el } = mount(reg, 'sui-view-chrome', { title: 'Git' })
    await flush()
    expect(shell['breadcrumbs-title'].textContent).toBe('Git')
    el.remove()
    expect(shell['breadcrumbs-title'].textContent).toBe('')
  })

  it('reflects title as attribute on host', () => {
    const { el } = mount(reg, 'sui-view-chrome', { title: 'Branches' })
    expect(el.getAttribute('title')).toBe('Branches')
  })

  it('reflects tone as attribute on host', () => {
    const { el } = mount(reg, 'sui-view-chrome', { tone: 'warning' })
    expect(el.getAttribute('tone')).toBe('warning')
  })

  it('reflects busy as attribute on host', () => {
    const { el } = mount(reg, 'sui-view-chrome', { busy: true })
    expect(el.hasAttribute('busy')).toBe(true)
  })
})
