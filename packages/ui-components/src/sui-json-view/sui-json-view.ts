import { controller, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-json-view.sui'

type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue }

function escape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function renderValue(v: JsonValue, key: string | null, level: number, expanded: boolean, depth: number): string {
  const keyPrefix = key !== null ? `<span class="key">"${escape(key)}"</span>: ` : ''
  if (v === null) return `${keyPrefix}<span class="null">null</span>`
  if (typeof v === 'string') return `${keyPrefix}<span class="s">"${escape(v)}"</span>`
  if (typeof v === 'number') return `${keyPrefix}<span class="n">${v}</span>`
  if (typeof v === 'boolean') return `${keyPrefix}<span class="b">${String(v)}</span>`
  const isArr = Array.isArray(v)
  const entries: Array<[string, JsonValue]> = isArr
    ? (v as JsonValue[]).map((x, i) => [String(i), x])
    : Object.entries(v as Record<string, JsonValue>)
  const open = expanded || level < depth
  const open1 = isArr ? '[' : '{'
  const close1 = isArr ? ']' : '}'
  const count = entries.length
  if (count === 0) return `${keyPrefix}${open1}${close1}`
  const children = entries
    .map(([k, val]) => `<div class="node">${renderValue(val, isArr ? null : k, level + 1, expanded, depth)}</div>`)
    .join('')
  return `<details${open ? ' open' : ''}>
<summary><span class="caret">▶</span>${keyPrefix}${open1}<span class="meta">${count} ${isArr ? 'items' : 'keys'}</span></summary>
<div class="branch">${children}</div>
${close1}</details>`
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    if (!host.hasAttribute('role')) host.setAttribute('role', 'tree')
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'JSON tree')

    const stopEffect = effect(() => {
      const el = scope.contentEl() as HTMLElement | null
      if (!el) return

      const data = scope.data()
      const expanded = scope.expanded()
      const depth = scope.depth()

      if (!data) {
        el.innerHTML = ''
        return
      }

      let parsed: JsonValue
      try {
        parsed = JSON.parse(data) as JsonValue
      } catch (e) {
        el.innerHTML = `<span class="error" part="error" role="alert">Invalid JSON: ${escape((e as Error).message)}</span>`
        return
      }

      el.innerHTML = `<div part="root">${renderValue(parsed, null, 0, expanded, depth)}</div>`
    })

    return () => stopEffect()
  },
})
