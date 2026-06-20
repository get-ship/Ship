// ssr-entry.ts — the SERVER ENTRY: render a synthesized SUI view to an HTML
// string INSIDE ship-js (a deno_core/V8 isolate), with route + data resolved
// over the host-dispatch wire — never the browser.
//
// It installs a minimal `globalThis.document` so the SAME synthesized render
// that runs in the browser (`createElement`, `appendChild`, `bindText`,
// `bindFor`, …) runs here unchanged and serializes the built tree to HTML —
// one isomorphic render artifact, not a second string-renderer.
//
// `import "ship"` is the one door: `call(op, args)` is the host-dispatch wire
// (`op_ship_call` → the host's `Dispatch`), `emit(v)` surfaces the result.

import { mount } from './page.render.ts'
import { call, emit } from 'ship'

// ── minimal server DOM ──────────────────────────────────────────────────────
// Covers what the synthesized render emits: text, comment (control-flow
// anchors), element (tag/attrs/class/style/children), and document fragments.
// Each node serializes itself to HTML. Event listeners are no-ops server-side.

const ESC: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }
const esc = (s: string): string => s.replace(/[&<>"]/g, (c) => ESC[c]!)
// Void elements never emit a closing tag.
const VOID = new Set([
  'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta',
  'param', 'source', 'track', 'wbr',
])

type SNode = TextNode | CommentNode | El

class TextNode {
  parentNode: El | Frag | null = null
  _t = ''
  get textContent(): string {
    return this._t
  }
  set textContent(v: unknown) {
    this._t = v == null ? '' : String(v)
  }
  toHTML(): string {
    return esc(this._t)
  }
}

class CommentNode {
  parentNode: El | Frag | null = null
  data: string
  // Mark control-flow anchors with nodeType 8 (Comment), as the runtime checks.
  nodeType = 8
  constructor(data: string) {
    this.data = data
  }
  toHTML(): string {
    return `<!--${this.data}-->`
  }
}

class ClassList {
  constructor(private el: El) {}
  add(...names: string[]): void {
    for (const n of names) this.el._classes.add(n)
  }
  remove(...names: string[]): void {
    for (const n of names) this.el._classes.delete(n)
  }
  contains(n: string): boolean {
    return this.el._classes.has(n)
  }
  toggle(n: string, force?: boolean): boolean {
    const on = force ?? !this.el._classes.has(n)
    if (on) this.el._classes.add(n)
    else this.el._classes.delete(n)
    return on
  }
}

class Style {
  props = new Map<string, string>()
  setProperty(k: string, v: string): void {
    this.props.set(k, v)
  }
  removeProperty(k: string): void {
    this.props.delete(k)
  }
}

class El {
  nodeType = 1
  tag: string
  parentNode: El | Frag | null = null
  attrs = new Map<string, string>()
  childNodes: SNode[] = []
  _classes = new Set<string>()
  classList: ClassList
  style = new Style()
  ownerDocument = g_document
  namespaceURI: string | null = null
  constructor(tag: string) {
    this.tag = tag
    this.classList = new ClassList(this)
  }
  setAttribute(name: string, value: unknown): void {
    if (name === 'class') {
      this._classes = new Set(String(value).split(/\s+/).filter(Boolean))
      return
    }
    this.attrs.set(name, value == null ? '' : String(value))
  }
  removeAttribute(name: string): void {
    if (name === 'class') this._classes.clear()
    else this.attrs.delete(name)
  }
  getAttribute(name: string): string | null {
    if (name === 'class') return [...this._classes].join(' ')
    return this.attrs.get(name) ?? null
  }
  appendChild<T extends SNode | Frag>(child: T): T {
    adopt(this, child)
    return child
  }
  insertBefore<T extends SNode>(node: T, ref: SNode | null): T {
    if (ref == null) return this.appendChild(node) as unknown as T
    const i = this.childNodes.indexOf(ref)
    detach(node)
    if (i < 0) this.childNodes.push(node)
    else this.childNodes.splice(i, 0, node)
    node.parentNode = this
    return node
  }
  removeChild<T extends SNode>(node: T): T {
    const i = this.childNodes.indexOf(node)
    if (i >= 0) this.childNodes.splice(i, 1)
    node.parentNode = null
    return node
  }
  addEventListener(): void {}
  removeEventListener(): void {}
  get textContent(): string {
    return this.childNodes.map((c) => c.textContent).join('')
  }
  set textContent(v: unknown) {
    const t = new TextNode()
    t.textContent = v
    this.childNodes = []
    this.appendChild(t)
  }
  toHTML(): string {
    const cls = this._classes.size ? ` class="${esc([...this._classes].join(' '))}"` : ''
    const styleStr = this.style.props.size
      ? ` style="${esc([...this.style.props].map(([k, v]) => `${k}: ${v}`).join('; '))}"`
      : ''
    const attrs = [...this.attrs].map(([k, v]) => ` ${k}="${esc(v)}"`).join('')
    const open = `<${this.tag}${cls}${styleStr}${attrs}>`
    if (VOID.has(this.tag)) return open
    const inner = this.childNodes.map((c) => c.toHTML()).join('')
    return `${open}${inner}</${this.tag}>`
  }
}

class Frag {
  childNodes: SNode[] = []
  appendChild<T extends SNode>(child: T): T {
    adopt(this, child)
    return child
  }
}

function detach(node: SNode): void {
  const p = node.parentNode
  if (p) {
    const i = p.childNodes.indexOf(node)
    if (i >= 0) p.childNodes.splice(i, 1)
    node.parentNode = null
  }
}

function adopt(parent: El | Frag, child: SNode | Frag): void {
  if (child instanceof Frag) {
    // Moving a fragment moves its children (DOM semantics).
    for (const c of [...child.childNodes]) {
      detach(c)
      parent.childNodes.push(c)
      c.parentNode = parent as El | Frag
    }
    child.childNodes = []
    return
  }
  detach(child)
  parent.childNodes.push(child)
  child.parentNode = parent
}

const g = globalThis as Record<string, unknown>
const g_document = {
  createElement: (tag: string): El => new El(tag),
  createElementNS: (_ns: string, tag: string): El => {
    const el = new El(tag)
    el.namespaceURI = _ns
    return el
  },
  createTextNode: (text: string): TextNode => {
    const t = new TextNode()
    t.textContent = text
    return t
  },
  createComment: (data: string): CommentNode => new CommentNode(data),
  createDocumentFragment: (): Frag => new Frag(),
}
g.document = g_document

// ── host-dispatch: resolve route server-side, park it for the route shim ─────
const route = (await call('route.current', {})) as { params?: Record<string, string> }
g.__shipServerRoute = route

// ── render the synthesized view to HTML ─────────────────────────────────────
const container = new El('__root__')
const dispose = mount(container as unknown as HTMLElement)

// Flush async `live()` sources before serializing (must-build #8): a
// `<data live source="api.x()">` resolves over the host-dispatch wire, which
// is an async op. Mounting renders the `loading` state synchronously; yielding
// lets the op resolve, the live signal update, and the bound effects re-fire
// against the server-DOM — so first paint carries the resolved value, not
// `loading`. Bounded so a never-resolving source can't hang the render.
for (let i = 0; i < 50; i++) {
  await Promise.resolve()
}

const html = container.childNodes.map((c) => c.toHTML()).join('')
dispose()

emit({ html, routeParams: route?.params ?? null })
