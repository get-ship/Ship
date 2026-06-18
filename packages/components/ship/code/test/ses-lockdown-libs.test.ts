// @vitest-environment node
//
// #133 slice 5 — do the code pack's heavy 3rd-party libs survive SES
// lockdown()? The CodeMirror/xterm dependencies live in @ship/code, so
// their lockdown-survival contract is pinned here.
//
// In the SES model the shell calls lockdownRealm() at boot (freezing
// intrinsics), then lazy-loads editor/terminal libs AFTER. The classic SES
// failure mode is a library that monkey-patches shared prototypes or relies on
// now-frozen intrinsics — that breaks at MODULE EVALUATION or core construction.
// This reproduces the realistic ordering: lockdown first, dynamic-import +
// construct second.
//
// ENV NOTE — this file pins `node`, NOT the shell's default jsdom: SES
// lockdown() is INCOMPATIBLE with jsdom, which installs a non-standard
// `ArrayBuffer` whose [[Prototype]] trips SES's intrinsic-permits check
// (`Unexpected [[Prototype]] at intrinsics.ArrayBuffer.__proto__`). The real
// browser the shell runs in has standard intrinsics, so this is a test-harness
// limitation, not a runtime one — but it means DOM-render-level verification
// (EditorView mounting, xterm canvas) must be done in a real browser
// (playwright), not jsdom. This test covers the lockdown-sensitive surface that
// IS reachable in node: module evaluation + DOM-free core construction.

import { beforeAll, describe, expect, it } from 'vitest'
import { lockdownRealm } from '../../../../deck/host/src/confine.ts'

describe('SES lockdown vs the shell 3rd-party editor libs (module + core)', () => {
  beforeAll(() => lockdownRealm())

  it('CodeMirror + xterm modules evaluate under lockdown (no load-time prototype mutation)', async () => {
    const cmState = await import('@codemirror/state')
    const cmView = await import('@codemirror/view')
    const xterm = await import('@xterm/xterm')
    expect(typeof cmState.EditorState).toBe('function')
    expect(typeof cmView.EditorView).toBe('function')
    expect(typeof xterm.Terminal).toBe('function')
  })

  it('CodeMirror core (EditorState + transaction) works under lockdown', async () => {
    const { EditorState } = await import('@codemirror/state')
    const st = EditorState.create({ doc: 'hello world' })
    const tr = st.update({ changes: { from: 5, insert: ' brave' } })
    expect(tr.state.doc.toString()).toBe('hello brave world')
  })
})
