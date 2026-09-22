<!--
  <ship-code-editor> — SUI island: CodeMirror 6 wrapped as a lazy-loaded
  web-component. Language extensions are dynamic-imported when `language`
  is set, keeping the base bundle light.

  Prop / event / method parity vs <ship-code-editor>
  ──────────────────────────────────────────────────────────────────────────
  Lit                            SUI                     Notes
  ──────────────────────────────────────────────────────────────────────────
  value (String, '')             value (string, '',      ✓ form-associated
                                   form-value)
  language (SupportedLang)       language (string, '')   ✓ dynamic import
  placeholder (String, '')       placeholder (string)    ✓
  readonly (Boolean, false)      readonly (bool,         ✓
                                   reflect)
  line-numbers (Boolean, true)   line-numbers (bool,     ✓ default true
                                   reflect)
  size (sm|md|lg, md)            size (union, md,        ✓
                                   reflect)
  tone (Tone, default)           tone (union, default,   ✓ focus ring
                                   reflect)
  theme (light|dark|auto, auto)  theme (union, auto,     ✓
                                   reflect)
  ──────────────────────────────────────────────────────────────────────────
  ship-change                    sui-change              ✓ debounced 150ms
  ship-save                      sui-save                ✓ Mod-s
  ship-cancel                    sui-cancel              ✓ Escape
  input                          input                   ✓ native bubbling
  ──────────────────────────────────────────────────────────────────────────
  setDiagnostics(diags)          host.setDiagnostics     ✓ imperative API
  focus()                        host.focus()            ✓
  ──────────────────────────────────────────────────────────────────────────
-->
<scope>
  props: {
    value = ''
    language = ''
    placeholder = ''
    readonly = false
    'line-numbers' = true
    size: 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' = 'default'
    theme: 'light' | 'dark' | 'auto' = 'auto'
  }
  style fill = { size({ w: max, min-w: 0, min-h: 0, h: max }).size(grow).overflow(hidden) }
</scope>

<controller>
// use: Action capture → attachCodeEditor (engine lives in .init.ts).

import type { Action } from '@ship/sail-runtime'
import type { Scope } from './ship-code-editor.sail'
import { attachCodeEditor } from './ship-code-editor.init'

export type { SuiDiagnostic, SuiCompletionSource } from './ship-code-editor.init'

export default controller((ctx: Scope, host?) => {
  const capture: Action<void> = (node) => {
    if (!node || !host) {
      console.warn('[ship-code-editor] use: ref is null — CodeMirror not initialized')
      return { destroy: () => {} }
    }
    return attachCodeEditor(node, host, {
      value: ctx.value,
      language: ctx.language,
      readonly: ctx.readonly,
      lineNumbers: ctx.lineNumbers,
      placeholder: ctx.placeholder,
    })
  }
  return { capture }
})
</controller>

<div use:capture style(fill)></div>
