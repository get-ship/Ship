/**
 * <ship-markdown-editor> — Lit web component wrapping TipTap v3. Edits
 * markdown; serializes via tiptap-markdown. Ships in its own lazy bundle
 * (package.json `bundle:editor`) — not in base ship-elements.js.
 *
 * Attributes: value, placeholder, size (sm|md|lg), tone, disabled, invalid,
 * rows (min visible, default 12), full-height, name, label.
 * Events: ship-change (debounced 150ms), input (bubbling for Datastar).
 * Form-associated: setFormValue reports current markdown.
 */

import { LitElement, html, nothing, svg } from 'lit'
import type { Editor } from '@tiptap/core'
import { shipMarkdownEditorStyles } from './ship-markdown-editor.styles'
import type { Size, Tone } from './util'

// NOTE: TipTap subpath packages export ESM via their `module` field. We
// dynamic-import to code-split the heavy graph away from registration.
type MarkdownEditor = Editor & { storage: { markdown?: { getMarkdown(): string } } }
type MarkMap = { bold: boolean; italic: boolean; code: boolean; h1: boolean
  h2: boolean; h3: boolean; bullet: boolean; ordered: boolean; task: boolean; link: boolean }

export class ShipMarkdownEditor extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  private editor: MarkdownEditor | null = null
  private editorHost: HTMLDivElement | null = null
  private changeTimer: ReturnType<typeof setTimeout> | null = null
  private syncTimer: ReturnType<typeof setTimeout> | null = null
  private suppressNextUpdate = false
  private marks: MarkMap = {
    bold: false, italic: false, code: false, h1: false, h2: false,
    h3: false, bullet: false, ordered: false, task: false, link: false,
  }

  constructor() { super(); this.internals = this.attachInternals() }

  static styles = shipMarkdownEditorStyles

  static properties = {
    value: { type: String },
    placeholder: { type: String },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    rows: { type: Number },
    fullHeight: { type: Boolean, reflect: true, attribute: 'full-height' },
    name: { type: String },
    label: { type: String },
  }

  value = ''
  placeholder = ''
  size: Size = 'md'
  tone?: Tone
  disabled = false
  invalid = false
  rows = 12
  fullHeight = false
  name = ''
  label = ''

  connectedCallback(): void {
    super.connectedCallback()
    this.defaultValue = this.value
    this.applyMinHeight()
  }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this.changeTimer) clearTimeout(this.changeTimer)
    if (this.syncTimer) clearTimeout(this.syncTimer)
    this.changeTimer = this.syncTimer = null
    if (this.editor) { try { this.editor.destroy() } catch { /* teardown race */ } }
    this.editor = null
  }

  formResetCallback(): void {
    this.value = this.defaultValue
    if (this.editor) this.editor.commands.setContent(this.defaultValue)
    this.setFormValue(this.value)
  }

  formStateRestoreCallback(state: string | null): void {
    const v = state ?? ''
    this.value = v
    if (this.editor) this.editor.commands.setContent(v)
  }

  private applyMinHeight() {
    const perRow = 28
    this.style.setProperty('--ed-min-h', `${Math.max(this.rows * perRow, 160)}px`)
  }

  private setFormValue(v: string) {
    if (typeof this.internals.setFormValue === 'function') this.internals.setFormValue(v)
  }

  firstUpdated(): void {
    this.editorHost = this.renderRoot.querySelector('[data-editor-host]')
    if (this.editorHost) void this.initEditor()
  }

  updated(changed: Map<string, unknown>): void {
    if (!this.editor) return
    if (changed.has('value')) {
      if (this.syncTimer) clearTimeout(this.syncTimer)
      this.syncTimer = setTimeout(() => this.syncExternalValue(), 50)
    }
    if (changed.has('disabled')) this.editor.setEditable(!this.disabled)
    if (changed.has('rows')) this.applyMinHeight()
    this.setFormValue(this.value)
  }

  private async initEditor(): Promise<void> {
    if (!this.editorHost) return
    const [core, starter, Placeholder, tableSuite, Link, Image, TaskList,
      TaskItem, Typography, CodeBlockLowlight, markdownMod, lowlightMod] = await Promise.all([
      import('@tiptap/core'),
      import('@tiptap/starter-kit').then(m => m.default ?? m),
      import('@tiptap/extension-placeholder').then(m => m.default ?? m),
      import('@tiptap/extension-table'),
      import('@tiptap/extension-link').then(m => m.default ?? m),
      import('@tiptap/extension-image').then(m => m.default ?? m),
      import('@tiptap/extension-task-list').then(m => m.default ?? m),
      import('@tiptap/extension-task-item').then(m => m.default ?? m),
      import('@tiptap/extension-typography').then(m => m.default ?? m),
      import('@tiptap/extension-code-block-lowlight').then(m => m.default ?? m),
      import('tiptap-markdown'),
      import('lowlight'),
    ])
    const lowlight = lowlightMod.createLowlight(lowlightMod.common)
    const { Table, TableRow, TableCell, TableHeader } = tableSuite as {
      Table: unknown; TableRow: unknown; TableCell: unknown; TableHeader: unknown
    }
    const { Markdown } = markdownMod
    const Starter = starter as { configure: (o: unknown) => unknown }
    const PH = Placeholder as { configure: (o: unknown) => unknown }
    const LinkExt = Link as { configure: (o: unknown) => unknown }
    const TaskItemExt = TaskItem as { configure: (o: unknown) => unknown }
    const TableExt = Table as { configure: (o: unknown) => unknown }
    const CodeBlock = CodeBlockLowlight as { configure: (o: unknown) => unknown }

    this.editor = new core.Editor({
      element: this.editorHost,
      extensions: [
        Starter.configure({ codeBlock: false, link: false }),
        PH.configure({ placeholder: this.placeholder ?? '' }),
        TableExt.configure({ resizable: false }),
        TableRow, TableCell, TableHeader,
        LinkExt.configure({ openOnClick: false, HTMLAttributes: { rel: 'noopener noreferrer' } }),
        Image, TaskList,
        TaskItemExt.configure({ nested: true }),
        Typography,
        CodeBlock.configure({ lowlight }),
        Markdown.configure({
          html: false, transformPastedText: true, transformCopiedText: true, linkify: false,
        }),
      ] as never,
      content: this.value,
      editable: !this.disabled,
      editorProps: {
        attributes: { spellcheck: 'false', role: 'textbox', 'aria-multiline': 'true' },
      },
      onUpdate: ({ editor: ed }) => this.onEditorUpdate(ed as MarkdownEditor),
      onSelectionUpdate: () => this.refreshMarks(),
      onTransaction: () => this.refreshMarks(),
    }) as MarkdownEditor
    this.refreshMarks()
  }

  private onEditorUpdate(ed: MarkdownEditor) {
    if (this.suppressNextUpdate) { this.suppressNextUpdate = false; return }
    const md = this.readMarkdown(ed)
    if (md === this.value) return
    this.value = md
    this.setFormValue(md)
    this.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    if (this.changeTimer) clearTimeout(this.changeTimer)
    this.changeTimer = setTimeout(() => {
      this.dispatchEvent(new CustomEvent('ship-change', {
        detail: { value: md }, bubbles: true, composed: true,
      }))
    }, 150)
  }

  private readMarkdown(ed: MarkdownEditor): string {
    const s = ed.storage?.markdown
    return s && typeof s.getMarkdown === 'function' ? s.getMarkdown() : ''
  }

  private syncExternalValue() {
    if (!this.editor) return
    const current = this.readMarkdown(this.editor)
    if (current === this.value) return
    this.suppressNextUpdate = true
    this.editor.commands.setContent(this.value)
  }

  private refreshMarks() {
    const ed = this.editor
    if (!ed) return
    this.marks = {
      bold: ed.isActive('bold'),
      italic: ed.isActive('italic'),
      code: ed.isActive('code'),
      h1: ed.isActive('heading', { level: 1 }),
      h2: ed.isActive('heading', { level: 2 }),
      h3: ed.isActive('heading', { level: 3 }),
      bullet: ed.isActive('bulletList'),
      ordered: ed.isActive('orderedList'),
      task: ed.isActive('taskList'),
      link: ed.isActive('link'),
    }
    this.requestUpdate()
  }

  private run(fn: (ed: MarkdownEditor) => void) {
    return (e: MouseEvent) => {
      e.preventDefault()
      if (this.editor && !this.disabled) fn(this.editor)
    }
  }

  private onLinkClick = (e: MouseEvent) => {
    e.preventDefault()
    if (!this.editor || this.disabled) return
    const prev = (this.editor.getAttributes('link').href as string | undefined) ?? ''
    const url = window.prompt('Link URL', prev); if (url === null) return
    const c = this.editor.chain().focus().extendMarkRange('link')
    url === '' ? c.unsetLink().run() : c.setLink({ href: url }).run()
  }

  private btn(name: string, label: string, active: boolean,
              onClick: (e: MouseEvent) => void, body: ReturnType<typeof html>, sc?: string) {
    return html`<button type="button" part="tb-${name}" class=${active ? 'is-active' : ''}
      aria-label=${label} aria-pressed=${active ? 'true' : 'false'}
      title=${sc ? `${label} (${sc})` : label} ?disabled=${this.disabled}
      @mousedown=${(e: MouseEvent) => e.preventDefault()} @click=${onClick}>${body}</button>`
  }

  private renderToolbar() {
    // `svg` tagged template keeps children in the SVG namespace so <path>
    // elements render under the renderer (plain `html` puts them in the
    // HTML namespace, where they have no layout box).
    const icon = (d: string) => html`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
      stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${
      d.split('|').map(p => svg`<path d=${p}></path>`)}</svg>`
    const txt = (t: string) => html`<span aria-hidden="true" style="font-weight:600;font-size:12px">${t}</span>`
    const m = this.marks
    return html`<div class="toolbar" role="toolbar" aria-label=${this.label || 'Editor toolbar'} part="toolbar">
      ${this.btn('bold', 'Bold', m.bold, this.run(ed => ed.chain().focus().toggleBold().run()),
        html`<span aria-hidden="true" style="font-weight:800;font-size:13px">B</span>`, 'Ctrl+B')}
      ${this.btn('italic', 'Italic', m.italic, this.run(ed => ed.chain().focus().toggleItalic().run()),
        html`<span aria-hidden="true" style="font-style:italic;font-weight:600;font-size:13px">I</span>`, 'Ctrl+I')}
      ${this.btn('code', 'Inline code', m.code, this.run(ed => ed.chain().focus().toggleCode().run()),
        icon('M16 18l6-6-6-6|M8 6l-6 6 6 6'))}
      <span class="divider" aria-hidden="true"></span>
      ${this.btn('h1', 'Heading 1', m.h1, this.run(ed => ed.chain().focus().toggleHeading({ level: 1 }).run()), txt('H1'))}
      ${this.btn('h2', 'Heading 2', m.h2, this.run(ed => ed.chain().focus().toggleHeading({ level: 2 }).run()), txt('H2'))}
      ${this.btn('h3', 'Heading 3', m.h3, this.run(ed => ed.chain().focus().toggleHeading({ level: 3 }).run()), txt('H3'))}
      <span class="divider" aria-hidden="true"></span>
      ${this.btn('bullet', 'Bullet list', m.bullet, this.run(ed => ed.chain().focus().toggleBulletList().run()),
        icon('M8 6h13|M8 12h13|M8 18h13|M3 6h.01|M3 12h.01|M3 18h.01'))}
      ${this.btn('ordered', 'Ordered list', m.ordered, this.run(ed => ed.chain().focus().toggleOrderedList().run()),
        icon('M10 6h11|M10 12h11|M10 18h11|M4 6h1v4|M4 10h2|M6 18H4c0-1 2-2 2-3s-1-1.5-2-1'))}
      ${this.btn('task', 'Task list', m.task, this.run(ed => ed.chain().focus().toggleTaskList().run()),
        icon('M9 11l3 3L22 4|M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11'))}
      <span class="divider" aria-hidden="true"></span>
      ${this.btn('link', 'Link', m.link, this.onLinkClick,
        icon('M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71|M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71'),
        'Ctrl+K')}
    </div>`
  }

  render() {
    return html`<div class="shell" part="shell"
      aria-invalid=${this.invalid ? 'true' : nothing}
      aria-disabled=${this.disabled ? 'true' : nothing}>
      ${this.renderToolbar()}
      <div class="surface" part="surface"><div data-editor-host part="editor"></div></div>
    </div>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-markdown-editor')) {
  customElements.define('ship-markdown-editor', ShipMarkdownEditor)
}

declare global {
  interface HTMLElementTagNameMap { 'ship-markdown-editor': ShipMarkdownEditor }
}
