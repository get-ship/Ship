import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import { bindRawHtml } from '../src/raw-html'

describe('bindRawHtml', () => {
  it('sets innerHTML from getter', () => {
    const el = document.createElement('div')
    const sig = signal('<b>Hi</b>')
    bindRawHtml(el, () => sig(), [sig])
    expect(el.innerHTML).toBe('<b>Hi</b>')
  })

  it('updates on dep change', () => {
    const el = document.createElement('div')
    const sig = signal('<i>a</i>')
    bindRawHtml(el, () => sig(), [sig])
    sig.set('<u>b</u>')
    expect(el.innerHTML).toBe('<u>b</u>')
  })

  it('null/undefined renders empty', () => {
    const el = document.createElement('div')
    const sig = signal<string | null>(null)
    bindRawHtml(el, () => sig() as string, [sig])
    expect(el.innerHTML).toBe('')
  })

  it('disposer stops updates', () => {
    const el = document.createElement('div')
    const sig = signal('a')
    const dispose = bindRawHtml(el, () => sig(), [sig])
    dispose()
    sig.set('b')
    expect(el.innerHTML).toBe('a')
  })
})
