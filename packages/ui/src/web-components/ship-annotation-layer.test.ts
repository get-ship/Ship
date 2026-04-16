import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-annotation-layer'
import type { ShipAnnotationLayer } from './ship-annotation-layer'
import type { AnnotationEvent } from '../utils/annotation-helpers'
import { cleanupTestDom } from './util'

async function mount(attrs = 'view-id="v1" mode="point" staging'): Promise<ShipAnnotationLayer> {
  document.body.insertAdjacentHTML(
    'beforeend',
    `<ship-annotation-layer ${attrs}><div id="child">content</div></ship-annotation-layer>`,
  )
  const el = document.body.lastElementChild as ShipAnnotationLayer
  await el.updateComplete
  return el
}

function stage(el: ShipAnnotationLayer, ev: AnnotationEvent): void {
  ;(el as unknown as { staged: AnnotationEvent[] }).staged = [ev]
}

function makeEvent(partial: Partial<AnnotationEvent> = {}): AnnotationEvent {
  return {
    id: '01HVCA9K6M5JVZ0000000000AA',
    kind: 'point',
    target: { view_id: 'v1', selector: '.btn' },
    created_at: '1970-01-01T00:00:00.000Z',
    author_kind: 'human',
    ...partial,
  }
}

describe('ship-annotation-layer', () => {
  afterEach(cleanupTestDom)

  it('reflects view-id and mode attributes', async () => {
    const el = await mount('view-id="xyz" mode="region"')
    expect(el.viewId).toBe('xyz')
    expect(el.mode).toBe('region')
  })

  it('defaults mode to off and staging to false', async () => {
    document.body.insertAdjacentHTML(
      'beforeend',
      '<ship-annotation-layer view-id="v1"></ship-annotation-layer>',
    )
    const el = document.body.lastElementChild as ShipAnnotationLayer
    await el.updateComplete
    expect(el.mode).toBe('off')
    expect(el.staging).toBe(false)
  })

  it('renders staged region markers', async () => {
    const el = await mount()
    stage(
      el,
      makeEvent({
        kind: 'region',
        target: { view_id: 'v1', rect: [10, 20, 100, 40] },
      }),
    )
    await el.updateComplete
    const marker = el.shadowRoot!.querySelector('ship-annotation-marker')
    expect(marker).toBeTruthy()
    expect(marker!.getAttribute('kind')).toBe('region')
  })

  it('emits ship-annotation-staged when confirming a pending point', async () => {
    const el = await mount()
    let got: AnnotationEvent | null = null
    el.addEventListener('ship-annotation-staged', (e) => {
      got = (e as CustomEvent).detail
    })
    type Internal = {
      _pending: { kind: 'point'; x: number; y: number; selector: string } | null
      confirmNote: (s: string) => void
    }
    const internal = el as unknown as Internal
    internal._pending = { kind: 'point', x: 10, y: 20, selector: '#child' }
    await el.updateComplete
    const ta = el.shadowRoot!.querySelector('textarea') as HTMLTextAreaElement
    ta.value = 'my note'
    const primary = el.shadowRoot!.querySelector('[data-primary]') as HTMLButtonElement
    primary.click()
    await el.updateComplete
    expect(got).not.toBeNull()
    expect(got!.note).toBe('my note')
    expect(got!.kind).toBe('point')
    expect(got!.target.view_id).toBe('v1')
    expect(got!.author_kind).toBe('human')
  })

  it('emits an AnnotationEvent JSON shape that matches the Rust contract (golden)', async () => {
    const el = await mount()
    type Internal = {
      _pending: { kind: 'point'; x: number; y: number; selector: string } | null
    }
    ;(el as unknown as Internal)._pending = {
      kind: 'point', x: 5, y: 7, selector: '#child',
    }
    await el.updateComplete
    let detail: AnnotationEvent | null = null
    el.addEventListener('ship-annotation-staged', (e) => {
      detail = (e as CustomEvent).detail
    })
    const ta = el.shadowRoot!.querySelector('textarea') as HTMLTextAreaElement
    ta.value = 'golden'
    ;(el.shadowRoot!.querySelector('[data-primary]') as HTMLButtonElement).click()
    await el.updateComplete
    expect(detail).not.toBeNull()
    const json = JSON.parse(JSON.stringify(detail))
    expect(Object.keys(json).sort()).toEqual(
      ['author_kind', 'created_at', 'id', 'kind', 'note', 'target'].sort(),
    )
    expect(json.author_kind).toBe('human')
    expect(json.kind).toBe('point')
    expect(json.target).toEqual({ view_id: 'v1', selector: '#child' })
    // Optional fields elide when unset (intent + selection).
    expect('intent' in json).toBe(false)
    expect('selection' in json).toBe(false)
  })

  it('POSTs staged items on send() and emits ship-annotation-sent', async () => {
    const el = await mount()
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ id: 'a' }), { status: 201 }),
    )
    globalThis.fetch = fetchMock as unknown as typeof fetch
    stage(el, makeEvent({ id: 'abc' }))
    let sentId: string | null = null
    el.addEventListener('ship-annotation-sent', (e) => {
      sentId = (e as CustomEvent).detail.id
    })
    await el.send()
    expect(fetchMock).toHaveBeenCalledTimes(1)
    const [url, init] = fetchMock.mock.calls[0]!
    expect(url).toBe('/api/runtime/feedback')
    expect((init as RequestInit).credentials).toBe('include')
    expect(sentId).toBe('abc')
  })

  it('emits ship-annotation-error on non-2xx and preserves the staged item', async () => {
    const el = await mount()
    globalThis.fetch = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ error: { code: 'invalid_id', message: 'bad' } }), {
        status: 422,
      }),
    ) as unknown as typeof fetch
    stage(el, makeEvent({ id: 'zzz' }))
    let err: { code: string } | null = null
    el.addEventListener('ship-annotation-error', (e) => {
      err = (e as CustomEvent).detail
    })
    await el.send()
    expect(err).not.toBeNull()
    expect(err!.code).toBe('invalid_id')
    expect(el.staged.length).toBe(1)
  })

  it('uses the custom endpoint attribute', async () => {
    const el = await mount('view-id="v" mode="point" staging endpoint="/custom/path"')
    const fetchMock = vi.fn().mockResolvedValue(new Response('{}', { status: 201 }))
    globalThis.fetch = fetchMock as unknown as typeof fetch
    stage(el, makeEvent())
    await el.send()
    expect(fetchMock.mock.calls[0]![0]).toBe('/custom/path')
  })

  it('clear() empties the staged array without sending', async () => {
    const el = await mount()
    const fetchMock = vi.fn()
    globalThis.fetch = fetchMock as unknown as typeof fetch
    stage(el, makeEvent())
    el.clear()
    expect(el.staged.length).toBe(0)
    expect(fetchMock).not.toHaveBeenCalled()
  })
})
