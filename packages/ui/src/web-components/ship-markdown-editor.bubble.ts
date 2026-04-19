/**
 * Selection-bubble and comment-composer render helpers for
 * <ship-markdown-editor>. Factored out to keep the component file under
 * the 300-line cap. These are pure render functions — all state lives
 * on the host element.
 */

import { html, nothing, svg } from 'lit'
import type { TemplateResult } from 'lit'
import type { MarkdownEditor } from './ship-markdown-editor.init'

export type BubbleMarks = { bold: boolean; italic: boolean; strike: boolean; code: boolean }
export type BubblePos = { top: number; left: number } | null
export type PendingComment = { text: string; value: string } | null

export interface BubbleHandlers {
  run: (fn: (ed: MarkdownEditor) => void) => (e: MouseEvent) => void
  onComment: () => void
  onGenerate: () => void
}

export interface BubbleRenderOptions {
  show: boolean
  pos: BubblePos
  pending: PendingComment
  marks: BubbleMarks
  allowComment: boolean
  allowGenerate: boolean
  handlers: BubbleHandlers
}

const svgIcon = (path: string) => html`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
  stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
  >${path.split('|').map(p => svg`<path d=${p}></path>`)}</svg>`

function fmtButton(active: boolean, label: string, klass: string, body: string,
                   onClick: (e: MouseEvent) => void): TemplateResult {
  return html`<button type="button" part="bubble-${label.toLowerCase()}"
    class=${active ? 'is-active' : ''} aria-label=${label} aria-pressed=${active ? 'true' : 'false'}
    title=${label} @mousedown=${(e: MouseEvent) => e.preventDefault()} @click=${onClick}
    ><span class=${klass} aria-hidden="true">${body}</span></button>`
}

function actionButton(label: string, onClick: () => void, path: string): TemplateResult {
  return html`<button type="button" part="bubble-${label.toLowerCase()}"
    aria-label=${label} title=${label}
    @mousedown=${(e: MouseEvent) => e.preventDefault()} @click=${onClick}>${svgIcon(path)}</button>`
}

export function renderBubble(o: BubbleRenderOptions): TemplateResult | typeof nothing {
  if (!o.show) return nothing
  const open = o.pos !== null && !o.pending
  const style = o.pos ? `top:${o.pos.top}px;left:${o.pos.left}px` : 'top:-9999px;left:-9999px'
  const { marks: m, handlers: h } = o
  const sep = html`<span class="sep" aria-hidden="true"></span>`
  const chatPath = 'M21 11.5a8.38 8.38 0 01-.9 3.8 8.5 8.5 0 01-7.6 4.7 8.38 8.38 0 01-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 01-.9-3.8 8.5 8.5 0 014.7-7.6 8.38 8.38 0 013.8-.9h.5a8.48 8.48 0 018 8v.5z'
  const sparklesPath = 'M12 3l1.9 4.6L18.5 9.5 14 12l-.5 5L12 13l-4 2 1.5-4.5L5.5 8l5-.4z|M20 3v4|M22 5h-4'
  return html`<div class="bubble" part="bubble" data-open=${open ? 'true' : 'false'}
    role="toolbar" aria-label="Selection actions" aria-hidden=${open ? 'false' : 'true'}
    style=${style} @mousedown=${(e: MouseEvent) => e.preventDefault()}>
    ${fmtButton(m.bold,   'Bold',   'glyph-bold',   'B',   h.run(ed => ed.chain().focus().toggleBold().run()))}
    ${fmtButton(m.italic, 'Italic', 'glyph-italic', 'I',   h.run(ed => ed.chain().focus().toggleItalic().run()))}
    ${fmtButton(m.strike, 'Strike', 'glyph-strike', 'S',   h.run(ed => ed.chain().focus().toggleStrike().run()))}
    ${fmtButton(m.code,   'Code',   'glyph-code',   '</>', h.run(ed => ed.chain().focus().toggleCode().run()))}
    ${(o.allowComment || o.allowGenerate) ? sep : nothing}
    ${o.allowComment  ? actionButton('Comment',  h.onComment,  chatPath)    : nothing}
    ${o.allowGenerate ? actionButton('Generate', h.onGenerate, sparklesPath) : nothing}
  </div>`
}

export interface ComposerRenderOptions {
  pending: PendingComment
  pos: BubblePos
  onInput: (e: Event) => void
  onKeyDown: (e: KeyboardEvent) => void
  onCancel: () => void
  onSubmit: () => void
}

export function renderComposer(o: ComposerRenderOptions): TemplateResult | typeof nothing {
  const p = o.pending; if (!p) return nothing
  const style = o.pos
    ? `top:${Math.max(o.pos.top + 10, 0)}px;left:${o.pos.left}px`
    : 'top:24px;left:50%'
  const trimmed = p.text.length > 140 ? p.text.slice(0, 140) + '…' : p.text
  return html`<div class="composer" part="composer" role="dialog" aria-label="Add comment"
    style=${style} @mousedown=${(e: MouseEvent) => e.stopPropagation()}>
    <div class="composer-quote">“${trimmed}”</div>
    <textarea data-composer-input rows="2" placeholder="Add your comment…"
      .value=${p.value} @input=${o.onInput} @keydown=${o.onKeyDown}></textarea>
    <div class="composer-actions">
      <button type="button" class="cancel" @click=${o.onCancel}>Cancel</button>
      <button type="button" class="submit" ?disabled=${!p.value.trim()}
        @click=${o.onSubmit}>Add comment <kbd>⌘↵</kbd></button>
    </div>
  </div>`
}
