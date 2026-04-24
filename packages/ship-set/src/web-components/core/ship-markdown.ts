/**
 * <ship-markdown>
 *
 * Minimal markdown renderer. Supports: headings (#..######), unordered
 * and ordered lists, fenced code blocks (```), inline code, links,
 * bold (**), italic (*), paragraphs, blockquotes.
 *
 * Attributes:
 *   - value: markdown source string
 */

import { LitElement, css, html } from 'lit'
import { unsafeHTML } from 'lit/directives/unsafe-html.js'
import { MONO } from './util'

function escape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function inline(s: string): string {
  let out = escape(s)
  out = out.replace(/`([^`]+)`/g, (_, c) => `<code>${c}</code>`)
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`)
  return out
}

export function renderMarkdown(src: string): string {
  const lines = src.replace(/\r\n/g, '\n').split('\n')
  const out: string[] = []
  let i = 0
  while (i < lines.length) {
    const line = lines[i]
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim()
      const buf: string[] = []
      i++
      while (i < lines.length && !/^```/.test(lines[i])) buf.push(lines[i++])
      i++
      out.push(`<pre data-lang="${escape(lang)}"><code>${escape(buf.join('\n'))}</code></pre>`)
      continue
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line)
    if (h) {
      out.push(`<h${h[1].length}>${inline(h[2])}</h${h[1].length}>`)
      i++
      continue
    }
    if (/^\s*[-*]\s+/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^\s*[-*]\s+/.test(lines[i])) {
        buf.push(`<li>${inline(lines[i].replace(/^\s*[-*]\s+/, ''))}</li>`)
        i++
      }
      out.push(`<ul>${buf.join('')}</ul>`)
      continue
    }
    if (/^\s*\d+\.\s+/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^\s*\d+\.\s+/.test(lines[i])) {
        buf.push(`<li>${inline(lines[i].replace(/^\s*\d+\.\s+/, ''))}</li>`)
        i++
      }
      out.push(`<ol>${buf.join('')}</ol>`)
      continue
    }
    if (/^>\s?/.test(line)) {
      const buf: string[] = []
      while (i < lines.length && /^>\s?/.test(lines[i])) { buf.push(lines[i].replace(/^>\s?/, '')); i++ }
      out.push(`<blockquote>${inline(buf.join(' '))}</blockquote>`)
      continue
    }
    if (line.trim() === '') { i++; continue }
    const buf: string[] = []
    while (i < lines.length && lines[i].trim() !== '' && !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(lines[i])) {
      buf.push(lines[i]); i++
    }
    out.push(`<p>${inline(buf.join(' '))}</p>`)
  }
  return out.join('\n')
}

export class ShipMarkdown extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      color: var(--foreground);
    }
    h1, h2, h3, h4, h5, h6 {
      font-family: var(--heading-font);
      font-weight: var(--font-weight-semibold);
      line-height: var(--leading-tight);
      margin: 1em 0 0.4em;
    }
    h1 { font-size: var(--text-2xl); }
    h2 { font-size: var(--text-xl); }
    h3 { font-size: var(--text-lg); }
    h4, h5, h6 { font-size: var(--text-md); }
    p { margin: 0.5em 0; }
    a { color: var(--primary); text-decoration: underline; text-underline-offset: 2px; }
    code {
      font-family: ${MONO};
      font-size: 0.92em;
      padding: 1px 5px;
      border-radius: var(--radius-sm);
      background: var(--muted);
      border: 1px solid var(--border);
    }
    pre {
      font-family: ${MONO};
      font-size: var(--text-sm);
      background: var(--muted);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      padding: 10px 12px;
      overflow-x: auto;
    }
    pre code { background: transparent; border: none; padding: 0; }
    blockquote {
      margin: 0.6em 0;
      padding: 4px 12px;
      border-left: 3px solid var(--border);
      color: var(--muted-foreground);
    }
    ul, ol { margin: 0.4em 0; padding-left: 22px; }
    li { margin: 2px 0; }
    strong { font-weight: var(--font-weight-semibold); }
  `

  static properties = {
    value: { type: String, reflect: true },
  }

  value = ''

  render() {
    return html`${unsafeHTML(renderMarkdown(this.value ?? ''))}`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-markdown')) {
  customElements.define('ship-markdown', ShipMarkdown)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-markdown': ShipMarkdown
  }
}
