import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-markdown-editor.sui'
import { createMarkdownEditor, readMarkdown, type MarkdownEditor } from './sui-markdown-editor.init'
import { injectEditorStyles } from './sui-markdown-editor.styles'
import { injectBubbleStyles } from './sui-markdown-editor.bubble.styles'

export type MarkMap = {
  bold: boolean; italic: boolean; strike: boolean; code: boolean
  h1: boolean; h2: boolean; h3: boolean; bullet: boolean
  ordered: boolean; task: boolean; link: boolean
}
export type BubblePos = { top: number; left: number } | null
export type PendingComment = { text: string } | null

const EMPTY_MARKS: MarkMap = {
  bold: false, italic: false, strike: false, code: false,
  h1: false, h2: false, h3: false, bullet: false, ordered: false, task: false, link: false,
}

function getSelectedText(ed: MarkdownEditor | null): string {
  if (!ed) return ''
  const { from, to } = ed.state.selection
  return ed.state.doc.textBetween(from, to, ' ').trim()
}

type ExtScope = Scope & {
  marks: Signal<MarkMap>; bubblePos: Signal<BubblePos>; pending: Signal<PendingComment>
  composerValue: Signal<string>; composerInputEl: Signal<HTMLTextAreaElement | null>
  shellEl: Signal<HTMLElement | null>; editorHost: Signal<HTMLElement | null>
  composerTop: Signal<string>; composerLeft: Signal<string>; pendingQuote: Signal<string>
  _editorRef: { current: MarkdownEditor | null }
}

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as ExtScope
    const hostEl = host as HTMLElement
    const _editorRef: { current: MarkdownEditor | null } = { current: null }
    const marks = signal<MarkMap>({ ...EMPTY_MARKS })
    const bubblePos = signal<BubblePos>(null)
    const pending = signal<PendingComment>(null)
    const composerValue = signal('')
    const shellEl = ref<HTMLElement>(null)
    const editorHost = ref<HTMLElement>(null)
    const composerInputEl = ref<HTMLTextAreaElement>(null)

    const composerTop = derived(() => {
      const pos = bubblePos()
      return pos ? `${Math.max(pos.top + 10, 0)}px` : '24px'
    })
    const composerLeft = derived(() => {
      const pos = bubblePos()
      return pos ? `${pos.left}px` : '50%'
    })
    const pendingQuote = derived(() => {
      const p = pending()
      if (!p) return ''
      return p.text.length > 140 ? p.text.slice(0, 140) + '…' : p.text
    })

    function disabled() { return (s.disabled as Signal<boolean>)() }

    function runCmd(fn: (ed: MarkdownEditor) => void) {
      const ed = _editorRef.current
      if (ed && !disabled()) fn(ed)
    }

    function emitFeedback(detail: Record<string, unknown>) {
      hostEl.dispatchEvent(new CustomEvent('sui-feedback', {
        detail, bubbles: true, composed: true,
      }))
    }

    return {
      _editorRef, marks, bubblePos, pending, composerValue,
      shellEl, editorHost, composerInputEl,
      composerTop, composerLeft, pendingQuote,

      onToolbarClick: (e: MouseEvent) => {
        const action = (e.target as Element).closest<HTMLElement>('[data-action]')?.dataset.action
        if (!action) return
        runCmd(ed => {
          const c = ed.chain().focus()
          switch (action) {
            case 'bold':    c.toggleBold().run(); break
            case 'italic':  c.toggleItalic().run(); break
            case 'code':    c.toggleCode().run(); break
            case 'h1':      c.toggleHeading({ level: 1 }).run(); break
            case 'h2':      c.toggleHeading({ level: 2 }).run(); break
            case 'h3':      c.toggleHeading({ level: 3 }).run(); break
            case 'bullet':  c.toggleBulletList().run(); break
            case 'ordered': c.toggleOrderedList().run(); break
            case 'task':    c.toggleTaskList().run(); break
          }
        })
      },

      onLinkClick: (e: MouseEvent) => {
        e.preventDefault()
        const ed = _editorRef.current
        if (!ed || disabled()) return
        const prev = (ed.getAttributes('link').href as string | undefined) ?? ''
        const url = window.prompt('Link URL', prev); if (url === null) return
        const c = ed.chain().focus().extendMarkRange('link')
        url === '' ? c.unsetLink().run() : c.setLink({ href: url }).run()
      },

      onBubbleFmt: (e: MouseEvent) => {
        const action = (e.target as Element).closest<HTMLElement>('[data-action]')?.dataset.action
        if (!action) return
        runCmd(ed => {
          const c = ed.chain().focus()
          switch (action) {
            case 'bold':   c.toggleBold().run(); break
            case 'italic': c.toggleItalic().run(); break
            case 'strike': c.toggleStrike().run(); break
            case 'code':   c.toggleCode().run(); break
          }
        })
      },

      onOpenComposer: () => {
        const text = getSelectedText(_editorRef.current)
        if (!text) return
        pending.set({ text }); composerValue.set(''); bubblePos.set(null)
      },

      onGenerate: () => {
        const text = getSelectedText(_editorRef.current)
        if (!text) return
        emitFeedback({ action: 'generate', selection: text })
        bubblePos.set(null)
      },

      onComposerKey: (e: KeyboardEvent) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
          e.preventDefault()
          const p = pending(); if (!p) return
          const comment = composerValue().trim(); if (!comment) return
          emitFeedback({ action: 'comment', selection: p.text, comment })
          pending.set(null); composerValue.set('')
        } else if (e.key === 'Escape') {
          e.preventDefault(); pending.set(null); composerValue.set('')
          _editorRef.current?.commands.focus()
        }
      },

      onCancel: () => {
        pending.set(null); composerValue.set('')
        _editorRef.current?.commands.focus()
      },

      onSubmit: () => {
        const p = pending(); if (!p) return
        const comment = composerValue().trim(); if (!comment) return
        emitFeedback({ action: 'comment', selection: p.text, comment })
        pending.set(null); composerValue.set('')
      },
    }
  },

  mount(state, host) {
    const s = state as unknown as ExtScope
    const shadow = host.shadowRoot
    if (shadow) { injectEditorStyles(shadow); injectBubbleStyles(shadow) }

    const editorHostEl = (s.editorHost as Signal<HTMLElement | null>)()
    const shellEl = (s.shellEl as Signal<HTMLElement | null>)()
    const disposers: Array<() => void> = []

    disposers.push(effect(() => {
      host.style.setProperty('--ed-min-h', `${Math.max((s.rows as Signal<number>)() * 28, 160)}px`)
    }))

    disposers.push(effect(() => {
      if ((s.pending as Signal<PendingComment>)()) {
        queueMicrotask(() => (s.composerInputEl as Signal<HTMLTextAreaElement | null>)()?.focus())
      }
    }))

    // Expose setters for testing — available before TipTap init so JSDOM tests work.
    const el = host as HTMLElement & {
      _setBubblePos?: (p: BubblePos) => void
      _setPending?: (p: PendingComment) => void
      _setMarks?: (m: MarkMap) => void
      _setComposerValue?: (v: string) => void
    }
    el._setBubblePos = (p) => (s.bubblePos as Signal<BubblePos>).set(p)
    el._setPending = (p) => {
      (s.pending as Signal<PendingComment>).set(p)
      if (p) (s.composerValue as Signal<string>).set('')
    }
    el._setMarks = (m) => (s.marks as Signal<MarkMap>).set(m)
    el._setComposerValue = (v) => (s.composerValue as Signal<string>).set(v)

    if (!editorHostEl) return () => { for (const d of disposers) d() }

    let changeTimer: ReturnType<typeof setTimeout> | null = null

    function refreshMarks() {
      const ed = s._editorRef.current; if (!ed) return
      ;(s.marks as Signal<MarkMap>).set({
        bold: ed.isActive('bold'), italic: ed.isActive('italic'),
        strike: ed.isActive('strike'), code: ed.isActive('code'),
        h1: ed.isActive('heading', { level: 1 }), h2: ed.isActive('heading', { level: 2 }),
        h3: ed.isActive('heading', { level: 3 }), bullet: ed.isActive('bulletList'),
        ordered: ed.isActive('orderedList'), task: ed.isActive('taskList'),
        link: ed.isActive('link'),
      })
    }

    function refreshBubble() {
      const ed = s._editorRef.current
      if (!ed || !(s.showSelectionToolbar as Signal<boolean>)()
        || (s.disabled as Signal<boolean>)()
        || (s.pending as Signal<PendingComment>)()) {
        ;(s.bubblePos as Signal<BubblePos>).set(null); return
      }
      const sel = ed.state.selection
      if (sel.empty) { (s.bubblePos as Signal<BubblePos>).set(null); return }
      try {
        const coords = ed.view.coordsAtPos(sel.from)
        const endCoords = ed.view.coordsAtPos(sel.to)
        const rect = shellEl?.getBoundingClientRect() ?? { top: 0, left: 0 }
        const top = Math.min(coords.top, endCoords.top) - rect.top - 42
        const left = (coords.left + endCoords.left) / 2 - rect.left
        ;(s.bubblePos as Signal<BubblePos>).set({ top: Math.max(-4, top), left: Math.max(20, left) })
      } catch { (s.bubblePos as Signal<BubblePos>).set(null) }
    }

    void (async () => {
      const ed = await createMarkdownEditor({
        element: editorHostEl,
        content: (s.value as Signal<string>)(),
        placeholder: (s.placeholder as Signal<string>)(),
        editable: !(s.disabled as Signal<boolean>)(),
        onUpdate: (editor) => {
          const md = readMarkdown(editor)
          if (md === (s.value as Signal<string>)()) return
          ;(s.value as Signal<string>).set(md)
          host.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
          if (changeTimer) clearTimeout(changeTimer)
          changeTimer = setTimeout(() => {
            host.dispatchEvent(new CustomEvent('sui-change', { detail: { value: md }, bubbles: true, composed: true }))
          }, 150)
        },
        onSelectionUpdate: () => { refreshMarks(); refreshBubble() },
        onTransaction: refreshMarks,
        onBlur: () => (s.bubblePos as Signal<BubblePos>).set(null),
      })
      s._editorRef.current = ed
      refreshMarks()
      host.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
      disposers.push(effect(() => {
        const v = (s.value as Signal<string>)()
        if (readMarkdown(ed) !== v) ed.commands.setContent(v)
      }))
      disposers.push(effect(() => { ed.setEditable(!(s.disabled as Signal<boolean>)()) }))
    })()

    return () => {
      if (changeTimer) clearTimeout(changeTimer)
      for (const d of disposers) d()
      try { s._editorRef.current?.destroy() } catch { /* teardown race */ }
      s._editorRef.current = null
    }
  },
})
