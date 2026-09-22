<!--
  <ship-markdown-editor> — form-associated TipTap v3 markdown editor.
  Inline toolbar, optional selection bubble + comment composer.
  Lazy-loads TipTap on mount. Styling via injectEditorStyles / injectBubbleStyles.
-->
<scope>
  props: {
    value = ''
    placeholder = ''
    size: 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    disabled = false
    invalid = false
    rows = 12
    'full-height' = false
    name = ''
    label = ''
    'show-selection-toolbar' = true
    'allow-comment' = true
    'allow-generate' = false
  }
  style shell = { position({ scheme: relative }).size({ w: max, min-w: 0, min-h: 0, h: max }).size(grow) }
  style toolbar = { pad({ x: 1, y: 1 }).border({ edge: b, width: 1, color: border }).bg(surface).overflow({ x: auto }) }
  style editorBox = { size({ w: max, min-w: 0, min-h: bind(edMinH) }).overflow({ y: auto }) }
  style editorFill = { size({ w: max, min-w: 0, min-h: 0, h: max }).size(grow).overflow({ y: auto }) }
  style pin = { position({ scheme: absolute, l: bind(pinX), t: bind(pinY) }) }
</scope>

<controller>
import type { Action } from '@ship/sail-runtime'
import { createMarkdownEditor, readMarkdown, type MarkdownEditor } from './ship-markdown-editor.init'
import { injectEditorStyles } from './ship-markdown-editor.styles'
import { injectBubbleStyles } from './ship-markdown-editor.bubble.styles'

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

function asHtml(node: Element | null): HTMLElement | null {
  return node instanceof HTMLElement ? node : null
}

type EditorProps = {
  value: () => string
  placeholder: () => string
  disabled: () => boolean
  invalid: () => boolean
  label: () => string
  rows: () => number
  fullHeight: () => boolean
  showSelectionToolbar: () => boolean
  allowComment: () => boolean
  allowGenerate: () => boolean
}

const ROW_PX = 24
const ROW_FLOOR = 28

export default controller((_ctx: EditorProps, host?: Element) => {
  const hostEl = asHtml(host ?? null)
  const _editorRef: { current: MarkdownEditor | null } = { current: null }
  const marks = signal<MarkMap>({ ...EMPTY_MARKS })
  const bubblePos = signal<BubblePos>(null)
  const pending = signal<PendingComment>(null)
  const composerValue = signal('')
  const shellEl = signal<Element | null>(null)
  const editorHost = signal<Element | null>(null)
  const composerInputEl = signal<Element | null>(null)

  const captureShell: Action<void> = (node) => {
    shellEl.set(node)
    return { destroy: () => shellEl.set(null) }
  }
  const captureEditor: Action<void> = (node) => {
    editorHost.set(node)
    return { destroy: () => editorHost.set(null) }
  }
  const captureComposer: Action<void> = (node) => {
    composerInputEl.set(node)
    return { destroy: () => composerInputEl.set(null) }
  }

  const bubblePin = derived(() => {
    const pos = bubblePos()
    return pos ? { x: pos.left, y: pos.top } : { x: -9999, y: -9999 }
  })
  const composerPin = derived(() => {
    const pos = bubblePos()
    return pos ? { x: pos.left, y: Math.max(pos.top + 10, 0) } : { x: 0, y: 24 }
  })
  const pendingQuote = derived(() => {
    const p = pending()
    if (!p) return ''
    return p.text.length > 140 ? p.text.slice(0, 140) + '…' : p.text
  })

  function isDisabled(): boolean {
    return _ctx.disabled() === true
  }

  function runCmd(fn: (ed: MarkdownEditor) => void): void {
    const ed = _editorRef.current
    if (ed && !isDisabled()) fn(ed)
  }

  function emitFeedback(detail: Record<string, unknown>): void {
    hostEl?.dispatchEvent(new CustomEvent('sail-feedback', {
      detail, bubbles: true, composed: true,
    }))
  }

  const edMinH = derived((): number => {
    const n = Number(_ctx.rows?.() ?? 12)
    const rows = Number.isFinite(n) && n > 0 ? n : 12
    return Math.max(ROW_FLOOR, rows * ROW_PX)
  })

  const onFmt = (action: string): void => {
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
  }

  const onLinkClick = (e: MouseEvent): void => {
    e.preventDefault()
    const ed = _editorRef.current
    if (!ed || isDisabled()) return
    const prev = (ed.getAttributes('link').href as string | undefined) ?? ''
    const ask = (globalThis as { prompt?: (m: string, d?: string) => string | null }).prompt
    const url = ask ? ask('Link URL', prev) : null
    if (url === null) return
    const c = ed.chain().focus().extendMarkRange('link')
    url === '' ? c.unsetLink().run() : c.setLink({ href: url }).run()
  }

  const onBubbleFmt = (e: MouseEvent): void => {
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
  }

  const onOpenComposer = (): void => {
    const text = getSelectedText(_editorRef.current)
    if (!text) return
    pending.set({ text }); composerValue.set(''); bubblePos.set(null)
  }

  const onGenerate = (): void => {
    const text = getSelectedText(_editorRef.current)
    if (!text) return
    emitFeedback({ action: 'generate', selection: text })
    bubblePos.set(null)
  }

  const onComposerInput = (e: { detail?: { value?: string }; target?: { value?: string } }): void => {
    composerValue.set(e.detail?.value ?? e.target?.value ?? '')
  }

  const onComposerKey = (e: KeyboardEvent): void => {
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
  }

  const onToolbarMouseDown = (e: MouseEvent): void => {
    e.preventDefault()
  }
  const onBubbleMouseDown = (e: MouseEvent): void => {
    e.preventDefault()
  }
  const onComposerMouseDown = (e: MouseEvent): void => {
    e.stopPropagation()
  }

  const onCancel = (): void => {
    pending.set(null); composerValue.set('')
    _editorRef.current?.commands.focus()
  }

  const onSubmit = (): void => {
    const p = pending(); if (!p) return
    const comment = composerValue().trim(); if (!comment) return
    emitFeedback({ action: 'comment', selection: p.text, comment })
    pending.set(null); composerValue.set('')
  }

  mount((host) => {
    injectEditorStyles(host)
    injectBubbleStyles(host)

    const disposers: Array<() => void> = []

    disposers.push(effect(() => {
      if (pending()) {
        queueMicrotask(() => asHtml(composerInputEl())?.focus())
      }
    }))

    const el = host as HTMLElement & {
      _setBubblePos?: (p: BubblePos) => void
      _setPending?: (p: PendingComment) => void
      _setMarks?: (m: MarkMap) => void
      _setComposerValue?: (v: string) => void
    }
    el._setBubblePos = (p) => bubblePos.set(p)
    el._setPending = (p) => {
      pending.set(p)
      if (p) composerValue.set('')
    }
    el._setMarks = (m) => marks.set(m)
    el._setComposerValue = (v) => composerValue.set(v)

    let changeTimer: ReturnType<typeof setTimeout> | null = null
    let disposed = false

    function refreshMarks(): void {
      const ed = _editorRef.current; if (!ed) return
      marks.set({
        bold: ed.isActive('bold'), italic: ed.isActive('italic'),
        strike: ed.isActive('strike'), code: ed.isActive('code'),
        h1: ed.isActive('heading', { level: 1 }), h2: ed.isActive('heading', { level: 2 }),
        h3: ed.isActive('heading', { level: 3 }), bullet: ed.isActive('bulletList'),
        ordered: ed.isActive('orderedList'), task: ed.isActive('taskList'),
        link: ed.isActive('link'),
      })
    }

    function refreshBubble(): void {
      const ed = _editorRef.current
      if (!ed || !_ctx.showSelectionToolbar() || isDisabled() || pending()) {
        bubblePos.set(null); return
      }
      const sel = ed.state.selection
      if (sel.empty) { bubblePos.set(null); return }
      try {
        const coords = ed.view.coordsAtPos(sel.from)
        const endCoords = ed.view.coordsAtPos(sel.to)
        const rect = shellEl()?.getBoundingClientRect() ?? { top: 0, left: 0 }
        const top = Math.min(coords.top, endCoords.top) - rect.top - 42
        const left = (coords.left + endCoords.left) / 2 - rect.left
        bubblePos.set({ top: Math.max(-4, top), left: Math.max(20, left) })
      } catch { bubblePos.set(null) }
    }

    queueMicrotask(() => {
      if (disposed) return
      const editorHostEl = asHtml(editorHost())
      if (!editorHostEl) return

      void (async () => {
        const ed = await createMarkdownEditor({
          element: editorHostEl,
          content: _ctx.value() ?? '',
          placeholder: _ctx.placeholder() ?? '',
          editable: !isDisabled(),
          onUpdate: (editor) => {
            const md = readMarkdown(editor)
            if (md === _ctx.value()) return
            ;(host as HTMLElement & { value: string }).value = md
            host.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
            if (changeTimer) clearTimeout(changeTimer)
            changeTimer = setTimeout(() => {
              host.dispatchEvent(new CustomEvent('sail-change', { detail: { value: md }, bubbles: true, composed: true }))
            }, 150)
          },
          onSelectionUpdate: () => { refreshMarks(); refreshBubble() },
          onTransaction: refreshMarks,
          onBlur: () => bubblePos.set(null),
        })
        if (disposed) { try { ed.destroy() } catch { /* teardown race */ } return }
        _editorRef.current = ed
        refreshMarks()
        host.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
        disposers.push(effect(() => {
          const v = _ctx.value() ?? ''
          if (readMarkdown(ed) !== v) ed.commands.setContent(v)
        }))
        disposers.push(effect(() => { ed.setEditable(!isDisabled()) }))
      })()
    })

    return () => {
      disposed = true
      if (changeTimer) clearTimeout(changeTimer)
      for (const d of disposers) d()
      try { _editorRef.current?.destroy() } catch { /* teardown race */ }
      _editorRef.current = null
    }
  })

  return {
    marks, bubblePos, pending, composerValue,
    captureShell, captureEditor, captureComposer,
    bubblePin, composerPin, pendingQuote,
    edMinH,
    fullHeight: () => _ctx.fullHeight() === true,
    onFmt, onLinkClick, onBubbleFmt,
    onToolbarMouseDown, onBubbleMouseDown, onComposerMouseDown,
    onOpenComposer, onGenerate, onComposerInput, onComposerKey, onCancel, onSubmit,
    disabled: () => isDisabled(),
    invalid: () => _ctx.invalid(),
    label: () => _ctx.label(),
    showSelectionToolbar: () => _ctx.showSelectionToolbar(),
    allowComment: () => _ctx.allowComment(),
    allowGenerate: () => _ctx.allowGenerate(),
  }
})
</controller>

<column use:captureShell
  aria-invalid={invalid() ? 'true' : null}
  aria-disabled={disabled() ? 'true' : null}
  style(shell)>
  <row gap="1" align="center" nowrap label={label() || 'Editor toolbar'} on:mousedown={onToolbarMouseDown} style(toolbar)>
    <button type="button" size="sm" variant="ghost" pressed={marks().bold} disabled={disabled()} label="Bold" on:click={() => onFmt('bold')}>B</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().italic} disabled={disabled()} label="Italic" on:click={() => onFmt('italic')}>I</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().code} disabled={disabled()} label="Inline code" on:click={() => onFmt('code')}>code</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().h1} disabled={disabled()} label="Heading 1" on:click={() => onFmt('h1')}>H1</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().h2} disabled={disabled()} label="Heading 2" on:click={() => onFmt('h2')}>H2</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().h3} disabled={disabled()} label="Heading 3" on:click={() => onFmt('h3')}>H3</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().bullet} disabled={disabled()} label="Bullet list" on:click={() => onFmt('bullet')}>•</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().ordered} disabled={disabled()} label="Ordered list" on:click={() => onFmt('ordered')}>1.</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().task} disabled={disabled()} label="Task list" on:click={() => onFmt('task')}>☑</button>
    <button type="button" size="sm" variant="ghost" pressed={marks().link} disabled={disabled()} label="Link" on:click={onLinkClick}>url</button>
  </row>
  <if {fullHeight()}>
    <div style(editorFill)>
      <div use:captureEditor></div>
    </div>
  {else =>
    <div style(editorBox)>
      <div use:captureEditor></div>
    </div>
  }</if>
  <if {showSelectionToolbar()}>
    <div role="toolbar" aria-label="Selection actions" data-md-bubble
      data-open={bubblePos() !== null && pending() === null ? 'true' : 'false'}
      aria-hidden={bubblePos() !== null && pending() === null ? 'false' : 'true'}
      use:pin={bubblePin()}
      style(pin)
      on:mousedown={onBubbleMouseDown}>
      <button type="button" data-is-active={marks().bold} aria-label="Bold" aria-pressed={marks().bold ? 'true' : 'false'} data-action="bold" on:click={onBubbleFmt}>B</button>
      <button type="button" data-is-active={marks().italic} aria-label="Italic" aria-pressed={marks().italic ? 'true' : 'false'} data-action="italic" on:click={onBubbleFmt}>I</button>
      <button type="button" data-is-active={marks().strike} aria-label="Strike" aria-pressed={marks().strike ? 'true' : 'false'} data-action="strike" on:click={onBubbleFmt}>S</button>
      <button type="button" data-is-active={marks().code} aria-label="Code" aria-pressed={marks().code ? 'true' : 'false'} data-action="code" on:click={onBubbleFmt}>code</button>
      <if {allowComment() || allowGenerate()}>
        <span aria-hidden="true" data-md-sep></span>
      </if>
      <if {allowComment()}>
        <button type="button" aria-label="Comment" on:click={onOpenComposer}>note</button>
      </if>
      <if {allowGenerate()}>
        <button type="button" aria-label="Generate" on:click={onGenerate}><icon name="sparkles" size="sm"></icon></button>
      </if>
    </div>
    <if {pending()}>
      <div role="dialog" aria-label="Add comment" data-md-composer
        use:pin={composerPin()}
        style(pin)
        on:mousedown={onComposerMouseDown}>
        <div data-md-composer-quote>"{pendingQuote()}"</div>
        <textarea rows="2" placeholder="Add your comment…" data-composer-input
          use:captureComposer
          value={composerValue()}
          on:input={onComposerInput}
          on:keydown={onComposerKey}></textarea>
        <div data-md-composer-actions>
          <button type="button" data-md-cancel on:click={onCancel}>Cancel</button>
          <button type="button" disabled={!composerValue().trim()} on:click={onSubmit}>Add comment <kbd>⌘↵</kbd></button>
        </div>
      </div>
    </if>
  </if>
</column>
