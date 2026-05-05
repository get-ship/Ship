import { controller, bindHtml } from '@ship/ui-runtime'
import type { Scope } from './sui-markdown.sui'

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

export default controller<Scope>({
  mount(state) {
    const scope = state as unknown as Scope
    const el = scope.contentEl() as HTMLElement | null
    if (!el) return () => {}
    return bindHtml(el, () => renderMarkdown(scope.value() ?? ''))
  },
})
