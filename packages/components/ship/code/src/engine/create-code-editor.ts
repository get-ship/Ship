// Headless reactive wrapper over CM6 EditorState/EditorView. Compartments hold
// the language / readonly / theme / line-numbers extensions so each can be
// reconfigured in place — no view rebuild, same DOM node. See build-spec Slice 1.
import {
  EditorState,
  EditorView,
  Compartment,
  keymap,
  lineNumbers,
  history,
  historyKeymap,
  defaultKeymap,
  indentUnit,
  autocompletion,
  search,
  searchKeymap,
  placeholder,
  type Extension,
} from '../vendor/codemirror/codemirror.mjs'
import { resolveLanguage } from './languages.js'
import { codeEditorTheme } from './theme.js'
import type { CodeEditor, CodeEditorOptions } from './types.js'

export function createCodeEditor(opts: CodeEditorOptions = {}): CodeEditor {
  const tabSize = opts.tabSize ?? 2
  const wantLineNumbers = opts.lineNumbers ?? true
  let dark = opts.dark ?? false
  let readonly = opts.readonly ?? false

  const languageC = new Compartment()
  const readonlyC = new Compartment()
  const themeC = new Compartment()
  const lineNumbersC = new Compartment()
  const wrapC = new Compartment()
  const placeholderC = new Compartment()

  // Guard onChange so it only fires on real doc mutations (setValue included),
  // not during the initial state construction.
  let ready = false

  const updateListener = EditorView.updateListener.of((update) => {
    if (ready && update.docChanged && opts.onChange) {
      opts.onChange(update.state.doc.toString())
    }
  })

  // onSave is mutable so the host (only available at mount, not construction)
  // can wire the Mod-s emitter after the engine is built. The keymap reads the
  // current holder on each invocation.
  let onSave = opts.onSave
  const saveKeymap = keymap.of([
    {
      key: 'Mod-s',
      run: (view) => {
        if (!onSave) return false
        onSave(view.state.doc.toString())
        return true
      },
    },
  ])

  const langExt = resolveLanguage(opts.language)

  const extensions: Extension[] = [
    lineNumbersC.of(wantLineNumbers ? lineNumbers() : []),
    history(),
    autocompletion(),
    search(),
    keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
    saveKeymap,
    indentUnit.of(' '.repeat(tabSize)),
    EditorState.tabSize.of(tabSize),
    languageC.of(langExt ?? []),
    readonlyC.of([EditorState.readOnly.of(readonly), EditorView.editable.of(!readonly)]),
    themeC.of(codeEditorTheme(dark)),
    wrapC.of(opts.lineWrap ? EditorView.lineWrapping : []),
    placeholderC.of(opts.placeholder ? placeholder(opts.placeholder) : []),
    updateListener,
  ]

  const state = EditorState.create({ doc: opts.doc ?? '', extensions })

  let view: EditorView | null = null

  function reconfigure(compartment: Compartment, extension: Extension): void {
    if (view) view.dispatch({ effects: compartment.reconfigure(extension) })
  }

  return {
    mount(parent: HTMLElement): void {
      view = new EditorView({ state, parent })
      ready = true
    },
    value(): string {
      return view ? view.state.doc.toString() : state.doc.toString()
    },
    setValue(v: string): void {
      if (!view) return
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } })
    },
    setLanguage(id: string): void {
      reconfigure(languageC, resolveLanguage(id) ?? [])
    },
    setReadonly(ro: boolean): void {
      readonly = ro
      reconfigure(readonlyC, [EditorState.readOnly.of(ro), EditorView.editable.of(!ro)])
    },
    isReadonly(): boolean {
      return readonly
    },
    setDark(d: boolean): void {
      dark = d
      reconfigure(themeC, codeEditorTheme(d))
    },
    setLineNumbers(on: boolean): void {
      reconfigure(lineNumbersC, on ? lineNumbers() : [])
    },
    setLineWrap(on: boolean): void {
      reconfigure(wrapC, on ? EditorView.lineWrapping : [])
    },
    setPlaceholder(text: string): void {
      reconfigure(placeholderC, text ? placeholder(text) : [])
    },
    setSave(cb: (value: string) => void): void {
      onSave = cb
    },
    focus(): void {
      view?.focus()
    },
    dispose(): void {
      view?.destroy()
      view = null
      ready = false
    },
  }
}
