/**
 * Main toolbar render helper for <ship-markdown-editor>. Split out of the
 * component so the component file stays under the 300-line cap. Pure
 * render function — all state and callbacks live on the host element.
 */

import { html, svg } from 'lit'
import type { TemplateResult } from 'lit'
import type { MarkdownEditor } from './ship-markdown-editor.init'

export type MarkMap = {
  bold: boolean; italic: boolean; strike: boolean; code: boolean
  h1: boolean; h2: boolean; h3: boolean; bullet: boolean
  ordered: boolean; task: boolean; link: boolean
}

export interface ToolbarOptions {
  marks: MarkMap
  label: string
  disabled: boolean
  run: (fn: (ed: MarkdownEditor) => void) => (e: MouseEvent) => void
  onLinkClick: (e: MouseEvent) => void
}

const icon = (d: string): TemplateResult =>
  html`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
    stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
    >${d.split('|').map(p => svg`<path d=${p}></path>`)}</svg>`

const txt = (t: string): TemplateResult =>
  html`<span aria-hidden="true" style="font-weight:600;font-size:12px">${t}</span>`

function btn(o: ToolbarOptions, name: string, label: string, active: boolean,
             onClick: (e: MouseEvent) => void, body: TemplateResult, sc?: string): TemplateResult {
  return html`<button type="button" part="tb-${name}" class=${active ? 'is-active' : ''}
    aria-label=${label} aria-pressed=${active ? 'true' : 'false'}
    title=${sc ? `${label} (${sc})` : label} ?disabled=${o.disabled}
    @mousedown=${(e: MouseEvent) => e.preventDefault()} @click=${onClick}>${body}</button>`
}

export function renderToolbar(o: ToolbarOptions): TemplateResult {
  const m = o.marks
  return html`<div class="toolbar" role="toolbar" aria-label=${o.label || 'Editor toolbar'} part="toolbar">
    ${btn(o, 'bold', 'Bold', m.bold, o.run(ed => ed.chain().focus().toggleBold().run()),
      html`<span aria-hidden="true" style="font-weight:800;font-size:13px">B</span>`, 'Ctrl+B')}
    ${btn(o, 'italic', 'Italic', m.italic, o.run(ed => ed.chain().focus().toggleItalic().run()),
      html`<span aria-hidden="true" style="font-style:italic;font-weight:600;font-size:13px">I</span>`, 'Ctrl+I')}
    ${btn(o, 'code', 'Inline code', m.code, o.run(ed => ed.chain().focus().toggleCode().run()),
      icon('M16 18l6-6-6-6|M8 6l-6 6 6 6'))}
    <span class="divider" aria-hidden="true"></span>
    ${btn(o, 'h1', 'Heading 1', m.h1, o.run(ed => ed.chain().focus().toggleHeading({ level: 1 }).run()), txt('H1'))}
    ${btn(o, 'h2', 'Heading 2', m.h2, o.run(ed => ed.chain().focus().toggleHeading({ level: 2 }).run()), txt('H2'))}
    ${btn(o, 'h3', 'Heading 3', m.h3, o.run(ed => ed.chain().focus().toggleHeading({ level: 3 }).run()), txt('H3'))}
    <span class="divider" aria-hidden="true"></span>
    ${btn(o, 'bullet', 'Bullet list', m.bullet, o.run(ed => ed.chain().focus().toggleBulletList().run()),
      icon('M8 6h13|M8 12h13|M8 18h13|M3 6h.01|M3 12h.01|M3 18h.01'))}
    ${btn(o, 'ordered', 'Ordered list', m.ordered, o.run(ed => ed.chain().focus().toggleOrderedList().run()),
      icon('M10 6h11|M10 12h11|M10 18h11|M4 6h1v4|M4 10h2|M6 18H4c0-1 2-2 2-3s-1-1.5-2-1'))}
    ${btn(o, 'task', 'Task list', m.task, o.run(ed => ed.chain().focus().toggleTaskList().run()),
      icon('M9 11l3 3L22 4|M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11'))}
    <span class="divider" aria-hidden="true"></span>
    ${btn(o, 'link', 'Link', m.link, o.onLinkClick,
      icon('M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71|M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71'),
      'Ctrl+K')}
  </div>`
}
