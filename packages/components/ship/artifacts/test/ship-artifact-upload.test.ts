import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-artifact-upload/ship-artifact-upload.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

// Returns a promise that resolves with the next matching event. Attach
// the listener FIRST, then drive the action — no real-clock timer.
function nextEvent(target: EventTarget, name: string): Promise<CustomEvent> {
  const { promise, resolve } = Promise.withResolvers<CustomEvent>()
  target.addEventListener(name, (e) => resolve(e as CustomEvent), { once: true })
  return promise
}

function makeFile(name: string, type: string): File {
  return new File([new Uint8Array([1, 2, 3])], name, { type })
}

describe('<ship-artifact-upload>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-artifact-upload')).toBeDefined()
  })

  it('renders a drop zone (sui-file-upload) + collection input + submit button', async () => {
    const { el } = mount(reg, 'ship-artifact-upload', {
      'collection-id': 'col-1',
    })
    await flush()
    const root = shadow(el)
    expect(root.querySelector('sui-file-upload')).not.toBeNull()
    expect(root.querySelector('[data-test="upload-submit"]')).not.toBeNull()
    expect(root.querySelector('[data-test="upload-collection-input"]')).not.toBeNull()
  })

  it('disables the submit button when no file has been selected', async () => {
    const { el } = mount(reg, 'ship-artifact-upload')
    await flush()
    const btn = shadow(el).querySelector('[data-test="upload-submit"]') as HTMLElement & { disabled?: boolean }
    expect(btn).not.toBeNull()
    expect(btn.disabled).toBe(true)
  })

  it('emits ship-artifact-upload { files, collection_id } on submit', async () => {
    const { el } = mount(reg, 'ship-artifact-upload', {
      'collection-id': 'col-1',
    })
    await flush()
    // Simulate file selection by dispatching sui-change on the embedded file-upload.
    const fu = shadow(el).querySelector('sui-file-upload') as HTMLElement
    expect(fu).not.toBeNull()
    const file = makeFile('hello.txt', 'text/plain')
    fu.dispatchEvent(new CustomEvent('sail-change', {
      detail: { files: [file] },
      bubbles: true,
      composed: true,
    }))
    await flush()
    const p = nextEvent(el, 'ship-artifact-upload')
    const submit = shadow(el).querySelector('[data-test="upload-submit"]') as HTMLElement
    submit.click()
    const evt = await p
    const detail = (evt as CustomEvent<{ files: File[]; collection_id: string }>).detail
    expect(detail.collection_id).toBe('col-1')
    expect(detail.files.length).toBe(1)
    expect(detail.files[0]?.name).toBe('hello.txt')
  })
})
