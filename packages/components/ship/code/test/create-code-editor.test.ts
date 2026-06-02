import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { createCodeEditor } from '../src/engine/create-code-editor.js'

describe('createCodeEditor (headless engine)', () => {
  let parent: HTMLDivElement

  beforeEach(() => {
    parent = document.createElement('div')
    document.body.appendChild(parent)
  })

  afterEach(() => {
    parent.remove()
  })

  it('constructs and mounts an EditorView into the parent', () => {
    const editor = createCodeEditor({ doc: 'hello' })
    editor.mount(parent)
    expect(parent.querySelector('.cm-editor')).not.toBeNull()
    editor.dispose()
  })

  it('round-trips value() / setValue()', () => {
    const editor = createCodeEditor({ doc: 'one' })
    editor.mount(parent)
    expect(editor.value()).toBe('one')
    editor.setValue('two\nthree')
    expect(editor.value()).toBe('two\nthree')
    editor.dispose()
  })

  it('fires onChange on a document edit (transaction), not on construction', () => {
    const seen: string[] = []
    const editor = createCodeEditor({ doc: 'a', onChange: (v) => seen.push(v) })
    editor.mount(parent)
    expect(seen).toEqual([])
    editor.setValue('ab')
    expect(seen).toEqual(['ab'])
    editor.dispose()
  })

  it('setLanguage reconfigures the same view + content nodes (compartment, no rebuild)', () => {
    const editor = createCodeEditor({ doc: 'const x = 1', language: 'json' })
    editor.mount(parent)
    const viewEl = parent.querySelector('.cm-editor')
    const contentEl = parent.querySelector('.cm-content')
    expect(contentEl).not.toBeNull()

    editor.setLanguage('typescript')
    // Same DOM nodes — the language compartment reconfigured in place. A view
    // rebuild would replace .cm-editor AND .cm-content; a reconfigure keeps both.
    expect(parent.querySelector('.cm-editor')).toBe(viewEl)
    expect(parent.querySelector('.cm-content')).toBe(contentEl)
    expect(editor.value()).toBe('const x = 1')

    // Switch back the other direction — still the same instance, doc intact.
    editor.setLanguage('json')
    expect(parent.querySelector('.cm-editor')).toBe(viewEl)
    expect(parent.querySelector('.cm-content')).toBe(contentEl)
    expect(editor.value()).toBe('const x = 1')
    editor.dispose()
  })

  it('setReadonly toggles editability and blocks user edits', () => {
    const editor = createCodeEditor({ doc: 'x' })
    editor.mount(parent)
    expect(editor.isReadonly()).toBe(false)

    editor.setReadonly(true)
    expect(editor.isReadonly()).toBe(true)
    // The contenteditable host reflects EditorView.editable(false) → not editable.
    const content = parent.querySelector('.cm-content') as HTMLElement
    expect(content.getAttribute('contenteditable')).toBe('false')

    // Attempt an actual user-style edit: dispatch a beforeinput insertText on the
    // content node. A read-only view must drop it — the doc stays unchanged.
    const before = editor.value()
    content.dispatchEvent(
      new InputEvent('beforeinput', {
        inputType: 'insertText',
        data: 'INJECTED',
        bubbles: true,
        cancelable: true,
      }),
    )
    expect(editor.value()).toBe(before)
    expect(editor.value()).toBe('x')

    editor.setReadonly(false)
    expect(editor.isReadonly()).toBe(false)
    expect(content.getAttribute('contenteditable')).toBe('true')
    editor.dispose()
  })

  it('placeholder renders .cm-placeholder for an empty doc and clears when set empty', () => {
    const editor = createCodeEditor({ doc: '', placeholder: 'Type code…' })
    editor.mount(parent)
    const ph = parent.querySelector('.cm-placeholder')
    expect(ph).not.toBeNull()
    expect(ph?.textContent).toBe('Type code…')
    // Reconfigure to empty → placeholder extension removed.
    editor.setPlaceholder('')
    expect(parent.querySelector('.cm-placeholder')).toBeNull()
    editor.dispose()
  })

  it('dispose() destroys the view and empties the parent', () => {
    const editor = createCodeEditor({ doc: 'x' })
    editor.mount(parent)
    expect(parent.querySelector('.cm-editor')).not.toBeNull()
    editor.dispose()
    expect(parent.querySelector('.cm-editor')).toBeNull()
  })
})
