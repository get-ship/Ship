import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-tabs (sail)', () => {
  beforeAll(async () => {
    // Three components must register together — children before parent so
    // ship-tabs sees them at mount.
    await loadSailComponent('ship-tab.set')
    await loadSailComponent('ship-tab-panel.set')
    await loadSailComponent('ship-tabs.set')
  }, 120_000)

  function makeTabs(activeName = 'one'): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-tabs active="${activeName}">
        <ship-tab name="one" label="First"></ship-tab>
        <ship-tab name="two" label="Second" count="3"></ship-tab>
        <ship-tab name="three" label="Third" disabled></ship-tab>
        <ship-tab-panel name="one">PANEL_ONE</ship-tab-panel>
        <ship-tab-panel name="two">PANEL_TWO</ship-tab-panel>
        <ship-tab-panel name="three">PANEL_THREE</ship-tab-panel>
      </ship-tabs>
    `
    document.body.appendChild(wrap)
    return wrap.querySelector('ship-tabs') as HTMLElement
  }

  it('renders one button per ship-tab child with role=tab', () => {
    const tabs = makeTabs()
    const buttons = tabs.querySelectorAll('button.tab')
    expect(buttons.length).toBe(3)
    for (const b of buttons) expect(b.getAttribute('role')).toBe('tab')
    tabs.parentElement!.remove()
  })

  it('reflects label / count / disabled from <ship-tab> attrs', () => {
    const tabs = makeTabs()
    const buttons = Array.from(tabs.querySelectorAll('button.tab'))
    expect(buttons[0]!.querySelector('.label')!.textContent).toBe('First')
    expect(buttons[1]!.querySelector('.label')!.textContent).toBe('Second')
    expect(buttons[1]!.querySelector('.count')!.textContent).toBe('3')
    // Third has no count.
    expect(buttons[2]!.querySelector('.count')).toBeNull()
    expect((buttons[2] as HTMLButtonElement).disabled).toBe(true)
    expect(buttons[2]!.getAttribute('aria-disabled')).toBe('true')
    tabs.parentElement!.remove()
  })

  it('shows only the active panel; hides the others', () => {
    const tabs = makeTabs('two')
    const panels = tabs.querySelectorAll('ship-tab-panel')
    expect(panels[0]!.hasAttribute('active')).toBe(false)
    expect(panels[1]!.hasAttribute('active')).toBe(true)
    expect(panels[2]!.hasAttribute('active')).toBe(false)
    tabs.parentElement!.remove()
  })

  it('clicking a tab activates it and emits ship-tab-change', () => {
    const tabs = makeTabs('one')
    const events: any[] = []
    tabs.addEventListener('ship-tab-change', (e: any) => events.push(e.detail))
    const buttons = Array.from(tabs.querySelectorAll('button.tab')) as HTMLButtonElement[]

    buttons[1]!.click()
    expect(tabs.getAttribute('active')).toBe('two')
    expect(events).toEqual([{ name: 'two' }])

    // Second panel now visible.
    const panels = tabs.querySelectorAll('ship-tab-panel')
    expect(panels[1]!.hasAttribute('active')).toBe(true)
    expect(panels[0]!.hasAttribute('active')).toBe(false)

    tabs.parentElement!.remove()
  })

  it('disabled tab cannot be activated by click', () => {
    const tabs = makeTabs('one')
    const events: any[] = []
    tabs.addEventListener('ship-tab-change', (e: any) => events.push(e.detail))
    const disabledBtn = tabs.querySelectorAll('button.tab')[2] as HTMLButtonElement
    disabledBtn.click()
    expect(tabs.getAttribute('active')).toBe('one')
    expect(events.length).toBe(0)
    tabs.parentElement!.remove()
  })

  it('arrow-key navigation cycles enabled tabs only', () => {
    const tabs = makeTabs('one')
    const tablist = tabs.querySelector('.tablist')!
    const press = (key: string) => {
      const ev = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true })
      tablist.dispatchEvent(ev)
    }
    press('ArrowRight')
    expect(tabs.getAttribute('active')).toBe('two')
    // Next would be 'three' but it's disabled — skip to 'one'.
    press('ArrowRight')
    expect(tabs.getAttribute('active')).toBe('one')
    press('End')
    // End jumps to the last enabled tab (two).
    expect(tabs.getAttribute('active')).toBe('two')
    press('Home')
    expect(tabs.getAttribute('active')).toBe('one')
    tabs.parentElement!.remove()
  })

  it('tab-panel exposes the tabpanel role and ID convention', () => {
    const tabs = makeTabs()
    const panels = tabs.querySelectorAll('ship-tab-panel')
    expect(panels[0]!.getAttribute('role')).toBe('tabpanel')
    expect(panels[0]!.id).toBe('ship-tabpanel-one')
    expect(panels[0]!.getAttribute('aria-labelledby')).toBe('ship-tab-one')
    tabs.parentElement!.remove()
  })

  // GAP: dynamic add/remove of <ship-tab> children at runtime is not
  // observed in v1. The Lit version uses a MutationObserver; Sail
  // can't naively replicate that because the {#for} renders into the
  // same observed subtree (infinite loop). Tracked in README.
})
