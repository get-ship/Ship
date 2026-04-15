import { afterEach, describe, expect, it } from 'vitest'
import './ship-annotation-list'
import type { ShipAnnotationList } from './ship-annotation-list'
import type { AnnotationEvent } from '../utils/annotation-helpers'
import { cleanupTestDom } from './util'

function makeEvent(id: string, note: string): AnnotationEvent {
  return {
    id,
    kind: 'point',
    target: { view_id: 'v1', selector: '.btn' },
    note,
    created_at: new Date(0).toISOString(),
    author_kind: 'human',
  }
}

async function mount(): Promise<ShipAnnotationList> {
  document.body.insertAdjacentHTML('beforeend', '<ship-annotation-list></ship-annotation-list>')
  const el = document.body.lastElementChild as unknown as ShipAnnotationList
  await el.updateComplete
  return el
}

describe('ship-annotation-list', () => {
  afterEach(cleanupTestDom)

  it('shows empty state when staged is empty', async () => {
    const el = await mount()
    expect(el.shadowRoot!.querySelector('[part="empty"]')).toBeTruthy()
  })

  it('renders one <li> per staged annotation', async () => {
    const el = await mount()
    el.staged = [makeEvent('a', 'first'), makeEvent('b', 'second')]
    await el.updateComplete
    const items = el.shadowRoot!.querySelectorAll('[part="item"]')
    expect(items.length).toBe(2)
  })

  it('shows the note content', async () => {
    const el = await mount()
    el.staged = [makeEvent('a', 'hello world')]
    await el.updateComplete
    const note = el.shadowRoot!.querySelector('.note')!
    expect(note.textContent).toBe('hello world')
  })

  it('falls back to (no note) when note missing', async () => {
    const el = await mount()
    const ev = makeEvent('a', '')
    delete ev.note
    el.staged = [ev]
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('.note')!.textContent).toBe('(no note)')
  })

  it('dispatches ship-annotation-remove with the id on button click', async () => {
    const el = await mount()
    el.staged = [makeEvent('xyz', 'hi')]
    await el.updateComplete
    let detail: { id: string } | null = null
    el.addEventListener('ship-annotation-remove', (e) => {
      detail = (e as CustomEvent).detail
    })
    const btn = el.shadowRoot!.querySelector('button') as HTMLButtonElement
    btn.click()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('xyz')
  })

  it('parses JSON via the staged attribute', async () => {
    document.body.insertAdjacentHTML(
      'beforeend',
      `<ship-annotation-list staged='${JSON.stringify([makeEvent('q', 'attr')])}'></ship-annotation-list>`,
    )
    const el = document.body.lastElementChild as unknown as ShipAnnotationList
    await el.updateComplete
    expect(el.staged.length).toBe(1)
    expect(el.staged[0]!.id).toBe('q')
  })

  it('shows the kind label', async () => {
    const el = await mount()
    el.staged = [makeEvent('a', 'n')]
    await el.updateComplete
    const kind = el.shadowRoot!.querySelector('.kind')!
    expect(kind.textContent).toBe('point')
  })
})
