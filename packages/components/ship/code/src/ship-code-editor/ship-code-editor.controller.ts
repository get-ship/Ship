// Paired controller for <ship-code-editor>.
//
// Follows the imperative-engine SUI pattern (sui-terminal / sui-draw):
//   1. bind:this={hostEl} → Signal<El|null> reached via state.$signals at mount.
//   2. createCodeEditor() built from the initial props in scope; the engine owns
//      its DOM inside hostEl (canvas/xterm precedent — NOT controller innerHTML).
//   3. onChange bumps a `version` signal; debounce(version, 200) drives the
//      debounced ship-code-change emit. onSave emits ship-code-save on Mod-s.
//   4. A MutationObserver on the reflected attrs (language/readonly/theme/
//      line-wrap) reconfigures the engine compartments in place (syncTool
//      precedent) — no view rebuild.
//   5. mount returns a disposer: editor.dispose() + observer.disconnect().
import { controller, signal, debounce, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-code-editor.sui'
import { createCodeEditor } from '../engine/create-code-editor.js'

export default controller({
  scope: ({ value, language, readonly, theme, lineNumbers, lineWrap, tabSize, placeholder }: ViewScope<Scope>) => {
    // bind:this target — the div CM6's EditorView mounts into.
    const hostEl = ref<HTMLDivElement>(null)
    const version = signal(0)
    const changeSrc = debounce(version, 200)
    const lastValue = signal(value())
    const editor = createCodeEditor({
      doc: value(),
      language: language(),
      readonly: readonly(),
      lineNumbers: lineNumbers(),
      lineWrap: lineWrap(),
      tabSize: tabSize(),
      placeholder: placeholder(),
      dark: theme() === 'dark',
      onChange: (v) => {
        lastValue.set(v)
        version.set(version() + 1)
      },
    })
    return { editor, version, changeSrc, lastValue, hostEl }
  },

  mount(state, host) {
    const hostEl = state.$signals.hostEl()
    if (!hostEl) {
      console.warn('[ship-code-editor] bind:this ref is null — editor not mounted')
      return
    }

    const editor = state.editor
    // onSave needs the live host to emit; wire it now that mount has the host.
    editor.setSave((v: string) => host.emit!('ship-code-save', { value: v }))
    editor.mount(hostEl)

    // Expose the headless engine on the host for imperative callers (the
    // sui-terminal `el.terminal` precedent) — read-only value/setValue access
    // without reaching into the shadow DOM.
    ;(host as HTMLElement & { editor?: typeof editor }).editor = editor

    // Reflected attrs → engine compartment reconfigure (no view rebuild).
    const syncAttrs = (): void => {
      const language = host.getAttribute('language') ?? ''
      const readonly = host.hasAttribute('readonly')
      const dark = host.getAttribute('theme') === 'dark'
      const lineWrap = host.hasAttribute('line-wrap')
      const placeholder = host.getAttribute('placeholder') ?? ''
      editor.setLanguage(language)
      if (editor.isReadonly() !== readonly) editor.setReadonly(readonly)
      editor.setDark(dark)
      editor.setLineWrap(lineWrap)
      editor.setPlaceholder(placeholder)
    }
    const observer = new MutationObserver(syncAttrs)
    observer.observe(host, {
      attributes: true,
      attributeFilter: ['language', 'readonly', 'theme', 'line-wrap', 'placeholder'],
    })

    // Debounced doc-change event: fires when the 200ms-debounced version ticks.
    const stopChange = effect(() => {
      const v = state.changeSrc
      if (v === 0) return
      host.emit!('ship-code-change', { value: state.lastValue })
    })

    return () => {
      observer.disconnect()
      stopChange()
      editor.dispose()
    }
  },
})
