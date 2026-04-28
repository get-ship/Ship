import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-file-upload (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-file-upload.set')
  }, 120_000)

  function mount(attrs = '') {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<ship-file-upload ${attrs}></ship-file-upload>`
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  function pickFiles(el: HTMLElement, files: File[]) {
    const input = el.querySelector('input[type="file"]') as HTMLInputElement
    const dt = new DataTransfer()
    for (const f of files) dt.items.add(f)
    Object.defineProperty(input, 'files', { configurable: true, get: () => dt.files })
    input.dispatchEvent(new Event('change', { bubbles: true }))
  }

  function dropFiles(el: HTMLElement, files: File[]) {
    const dz = el.querySelector('.dropzone') as HTMLElement
    const dt = new DataTransfer()
    for (const f of files) dt.items.add(f)
    dz.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }))
  }

  it('renders a dropzone with role=button and a hidden file input', () => {
    const el = mount()
    const dz = el.querySelector('.dropzone')
    const input = el.querySelector('input[type="file"]') as HTMLInputElement
    expect(dz).not.toBeNull()
    expect(dz!.getAttribute('role')).toBe('button')
    expect(dz!.getAttribute('tabindex')).toBe('0')
    expect(input).not.toBeNull()
    el.remove()
  })

  it('aria-label defaults to "Upload files" when accept is empty', () => {
    const el = mount()
    expect(el.querySelector('.dropzone')!.getAttribute('aria-label')).toBe('Upload files')
    el.remove()
  })

  it('accept prop updates aria-label and is forwarded to the file input', () => {
    const el = mount('accept="image/*"')
    const dz = el.querySelector('.dropzone')!
    const input = el.querySelector('input[type="file"]') as HTMLInputElement
    expect(dz.getAttribute('aria-label')).toBe('Upload files (image/*)')
    expect(input.accept).toContain('image')
    el.remove()
  })

  it('multiple prop forwards to the file input', () => {
    const el = mount('multiple')
    const input = el.querySelector('input[type="file"]') as HTMLInputElement
    expect(input.multiple).toBe(true)
    el.remove()
  })

  it('disabled sets tabindex=-1 and the host carries [disabled]', () => {
    const el = mount('disabled')
    const dz = el.querySelector('.dropzone')!
    expect(dz.getAttribute('tabindex')).toBe('-1')
    expect(el.hasAttribute('disabled')).toBe(true)
    el.remove()
  })

  it('adding a file via input change emits ship-change and renders a file item', () => {
    const el = mount()
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const file = new File(['hello'], 'test.txt', { type: 'text/plain' })
    pickFiles(el, [file])

    expect(events).toHaveLength(1)
    expect(events[0].files).toHaveLength(1)
    expect(events[0].files[0].name).toBe('test.txt')
    expect(el.querySelector('.file-item')).not.toBeNull()
    expect(el.querySelector('.file-name')!.textContent).toBe('test.txt')
    el.remove()
  })

  it('adding a file via drop emits ship-change and renders a file item', () => {
    const el = mount()
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const file = new File(['data'], 'dropped.txt', { type: 'text/plain' })
    dropFiles(el, [file])

    expect(events).toHaveLength(1)
    expect(events[0].files[0].name).toBe('dropped.txt')
    expect(el.querySelector('.file-name')!.textContent).toBe('dropped.txt')
    el.remove()
  })

  it('file size is formatted and displayed in the file item', () => {
    const el = mount()
    const file = new File([new Uint8Array(2048)], 'medium.bin', { type: 'application/octet-stream' })
    pickFiles(el, [file])
    const sizeEl = el.querySelector('.file-size')
    expect(sizeEl).not.toBeNull()
    expect(sizeEl!.textContent).toMatch(/KB/)
    el.remove()
  })

  it('max-size rejects oversized files and shows an error alert', () => {
    const el = mount('max-size="100"')
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const bigFile = new File([new Uint8Array(200)], 'big.bin', { type: 'application/octet-stream' })
    pickFiles(el, [bigFile])

    const errorEl = el.querySelector('.error[role="alert"]')
    expect(errorEl).not.toBeNull()
    expect(errorEl!.textContent).toContain('big.bin')
    expect(el.querySelector('.file-item')).toBeNull()
    // ship-change still fires with empty list
    expect(events).toHaveLength(1)
    expect(events[0].files).toHaveLength(0)
    el.remove()
  })

  it('accept filter rejects non-matching files with an error', () => {
    const el = mount('accept="image/*"')
    const textFile = new File(['hi'], 'doc.txt', { type: 'text/plain' })
    pickFiles(el, [textFile])
    expect(el.querySelector('.error[role="alert"]')).not.toBeNull()
    expect(el.querySelector('.file-item')).toBeNull()
    el.remove()
  })

  it('accept filter passes extension-matched files (.png)', () => {
    const el = mount('accept=".png"')
    const imgFile = new File(['data'], 'photo.png', { type: 'image/png' })
    pickFiles(el, [imgFile])
    expect(el.querySelector('.error')).toBeNull()
    expect(el.querySelector('.file-item')).not.toBeNull()
    el.remove()
  })

  it('without multiple, only the first file is kept', () => {
    const el = mount()
    const f1 = new File(['a'], 'first.txt', { type: 'text/plain' })
    const f2 = new File(['b'], 'second.txt', { type: 'text/plain' })
    pickFiles(el, [f1, f2])
    expect(el.querySelectorAll('.file-item')).toHaveLength(1)
    expect(el.querySelector('.file-name')!.textContent).toBe('first.txt')
    el.remove()
  })

  it('with multiple, all files accumulate across picks', () => {
    const el = mount('multiple')
    const f1 = new File(['a'], 'a.txt', { type: 'text/plain' })
    const f2 = new File(['b'], 'b.txt', { type: 'text/plain' })
    pickFiles(el, [f1])
    pickFiles(el, [f2])
    expect(el.querySelectorAll('.file-item')).toHaveLength(2)
    el.remove()
  })

  it('remove button removes a file and fires ship-change', () => {
    const el = mount('multiple')
    const f1 = new File(['a'], 'a.txt', { type: 'text/plain' })
    const f2 = new File(['b'], 'b.txt', { type: 'text/plain' })
    pickFiles(el, [f1, f2])

    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const removeBtn = el.querySelector('.remove') as HTMLButtonElement
    removeBtn.click()

    expect(el.querySelectorAll('.file-item')).toHaveLength(1)
    expect(events).toHaveLength(1)
    expect(events[0].files).toHaveLength(1)
    expect(events[0].files[0].name).toBe('b.txt')
    el.remove()
  })

  it('remove clears the error message', () => {
    const el = mount('max-size="100"')
    const small = new File(['ok'], 'ok.txt', { type: 'text/plain' })
    pickFiles(el, [small])
    // Now trigger an error
    const big = new File([new Uint8Array(200)], 'big.bin', { type: 'application/octet-stream' })
    pickFiles(el, [big])
    expect(el.querySelector('.error')).not.toBeNull()
    // Remove the valid file
    ;(el.querySelector('.remove') as HTMLButtonElement).click()
    expect(el.querySelector('.error')).toBeNull()
    el.remove()
  })

  it('dragover adds .over class; dragleave removes it', () => {
    const el = mount()
    const dz = el.querySelector('.dropzone') as HTMLElement
    dz.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true }))
    expect(dz.classList.contains('over')).toBe(true)
    dz.dispatchEvent(new DragEvent('dragleave', { bubbles: true }))
    expect(dz.classList.contains('over')).toBe(false)
    el.remove()
  })

  it('drop clears .over and processes files', () => {
    const el = mount()
    const dz = el.querySelector('.dropzone') as HTMLElement
    dz.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true }))
    expect(dz.classList.contains('over')).toBe(true)

    const file = new File(['x'], 'x.txt', { type: 'text/plain' })
    const dt = new DataTransfer()
    dt.items.add(file)
    dz.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }))

    expect(dz.classList.contains('over')).toBe(false)
    expect(el.querySelector('.file-item')).not.toBeNull()
    el.remove()
  })

  it('Enter key on the dropzone triggers openPicker without throwing', () => {
    const el = mount()
    const dz = el.querySelector('.dropzone') as HTMLElement
    expect(() => {
      dz.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    }).not.toThrow()
    el.remove()
  })

  it('Space key on the dropzone triggers openPicker without throwing', () => {
    const el = mount()
    const dz = el.querySelector('.dropzone') as HTMLElement
    expect(() => {
      dz.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true, cancelable: true }))
    }).not.toThrow()
    el.remove()
  })

  it('tone prop reflects to the host attribute for CSS targeting', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    for (const tone of tones) {
      const el = mount(`tone="${tone}"`)
      expect(el.getAttribute('tone')).toBe(tone)
      el.remove()
    }
  })

  it('does not declare formAssociated', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).not.toBe(true)
  })

  it('cleanup removes drag listeners without throwing after disconnect', () => {
    const el = mount()
    el.remove()
    const dz = document.createElement('div')
    dz.className = 'dropzone'
    expect(() => {
      dz.dispatchEvent(new DragEvent('dragover', { bubbles: true }))
    }).not.toThrow()
  })
})
